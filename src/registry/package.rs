use crate::{helpers, project, MESSAGES};

use colored::Colorize;
use flate2::write::GzEncoder;
use flate2::Compression;
use global_placeholders::global;
use indicatif::{ProgressBar, ProgressStyle};
use macros_rs::{fmtstr, str, string, ternary};
use std::fs::File;

#[derive(Debug, serde::Deserialize)]
struct AuthFile {
    token: String,
    access: String,
}

#[derive(Debug, serde::Deserialize)]
struct Response {
    message: serde_json::Value,
}

fn remove_tar(file: &str) {
    if let Err(_) = std::fs::remove_file(file) {
        eprintln!(" {}", MESSAGES.get("tar_error").unwrap().bright_red());
        std::process::exit(1);
    }
}

fn write_tar(file_name: &String) -> Result<(), std::io::Error> {
    let current_dir = std::env::current_dir().expect(MESSAGES.get("tar_error").unwrap());
    log::info!("creating file: {}", file_name);
    let tar_gz = File::create(file_name)?;
    let enc = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = tar::Builder::new(enc);

    tar.append_dir_all(".", format!("{}", current_dir.display()))?;
    Ok(())
}

pub fn publish() {
    let name = global!("vendor.name");
    let registry_link = global!("vendor.registry");

    match home::home_dir() {
        Some(path) => {
            if !std::path::Path::new(fmtstr!("{}/.{name}/temp", path.display())).is_dir() {
                std::fs::create_dir_all(format!("{}/.{name}/temp", path.display())).unwrap();
                println!("created {}/.{name}", path.display());
            }

            let package = project::package::read();
            let client = reqwest::blocking::Client::new();
            let file_name = format!("{}/.{name}/temp/{}.tgz", path.display(), package.info.name.replace("/", ":"));

            if std::path::Path::new(&file_name).is_file() {
                remove_tar(&file_name);
            }

            let auth = match std::fs::read_to_string(format!("{}/.{name}/credentials/{}].json", path.display(), registry_link.replace("://", "["))) {
                Ok(content) => match serde_json::from_str::<AuthFile>(&content) {
                    Ok(json) => json,
                    Err(_) => {
                        eprintln!("{} {}", "✖".red(), MESSAGES.get("publish_error_login").unwrap().bright_red());
                        std::process::exit(1);
                    }
                },
                Err(_) => {
                    eprintln!("{} {}", "✖".red(), MESSAGES.get("publish_error_login").unwrap().bright_red());
                    std::process::exit(1);
                }
            };

            println!(
                "{} {}@{}",
                MESSAGES.get("publish_msg").unwrap().bright_yellow(),
                format!("{}", package.info.name).bold(),
                format!("{}", package.info.version).bold()
            );

            let pb = ProgressBar::new_spinner();
            pb.enable_steady_tick(std::time::Duration::from_millis(80));
            pb.set_style(ProgressStyle::with_template("{spinner:.yellow} {msg}").unwrap().tick_strings(&[
                "[    ]", "[=   ]", "[==  ]", "[=== ]", "[ ===]", "[  ==]", "[   =]", "[    ]", "[   =]", "[  ==]", "[ ===]", "[====]", "[=== ]", "[==  ]", "[=   ]", "",
            ]));
            pb.set_message(string!(MESSAGES.get("publish_wait").unwrap()));

            if let Err(err) = write_tar(&file_name) {
                eprintln!("{} {}", "✖".red(), MESSAGES.get("publish_error_generic").unwrap().bright_red());
                eprintln!(" {} {}", "-".bright_red(), err.to_string().bright_red());
                remove_tar(&file_name);
                std::process::exit(1);
            }

            let form = reqwest::blocking::multipart::Form::new()
                .text("group", "local")
                .text("access", auth.access)
                .text("url", package.info.url)
                .text("name", package.info.name)
                .text("index", package.info.index)
                .text("author", package.info.author)
                .text("version", package.info.version)
                .text("license", package.info.license)
                .text("repository", package.info.repository)
                .text("description", package.info.description)
                .text("dependencies", format!("{:?}", package.dependencies))
                .text("visibility", ternary!(package.registry.public, "public", "private"))
                .file("tarball", &file_name)
                .unwrap();

            let response = client
                .post(format!("{registry_link}/api/v{}/create", env!("CARGO_PKG_VERSION").split(".").collect::<Vec<&str>>().join("")))
                .multipart(form)
                .header(reqwest::header::AUTHORIZATION, reqwest::header::HeaderValue::from_static(str!(auth.token.clone())))
                .send();

            match response {
                Ok(response) => {
                    match serde_json::from_str::<Response>(&response.text().unwrap()) {
                        Ok(json) => {
                            if &json.message["created"].to_string() == "null" {
                                let error = json.message["error"].to_string().clone();
                                pb.finish_with_message(format!(
                                    "\x08{} {}",
                                    "✖".red(),
                                    format!(
                                        "{}\n - {}",
                                        MESSAGES.get("publish_error").unwrap(),
                                        ternary!(helpers::trim_start_end(&error) == "ul", MESSAGES.get("publish_error_token").unwrap(), helpers::trim_start_end(&error))
                                    )
                                    .bright_red()
                                ));
                                remove_tar(&file_name);
                                std::process::exit(1);
                            } else {
                                pb.finish_with_message(format!(
                                    "\x08{} {}",
                                    "✔".green(),
                                    format!("{}{}", MESSAGES.get("publish_done").unwrap(), &json.message["created"]).bright_green()
                                ));
                                remove_tar(&file_name);
                            }
                        }
                        Err(_) => {
                            eprint!("\r{} {}\n", "✖".red(), MESSAGES.get("publish_error_generic").unwrap().bright_red());
                            remove_tar(&file_name);
                            std::process::exit(1);
                        }
                    };
                }
                Err(err) => {
                    eprint!("\r{} {}\n", "✖".red(), format!("{}: {}", MESSAGES.get("publish_error").unwrap(), err.to_string()).bright_red());
                    remove_tar(&file_name);
                    std::process::exit(1);
                }
            };
        }
        None => {
            eprintln!("{}", MESSAGES.get("home_error").unwrap().red());
            std::process::exit(1);
        }
    }
}
