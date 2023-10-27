use crate::{helpers, MESSAGES};
use colored::Colorize;
use global_placeholders::global;
use indicatif::{ProgressBar, ProgressStyle};
use inquire::{min_length, Password, PasswordDisplayMode, Text};
use std::io::Write;

#[derive(Debug, serde::Deserialize)]
struct Record {
    id: String,
}

#[derive(Debug, serde::Deserialize)]
struct Response {
    token: String,
    record: Record,
}

pub fn login() {
    let name = global!("vendor.name");
    let registry_link = global!("vendor.registry");

    match home::home_dir() {
        Some(path) => {
            if !helpers::Exists::folder(format!("{}/.{name}", path.display())).unwrap() {
                std::fs::create_dir_all(format!("{}/.{name}", path.display())).unwrap();
                println!("created {}/.{name}", &path.display());
            }

            if !helpers::Exists::folder(format!("{}/.{name}/credentials", path.display())).unwrap() {
                std::fs::create_dir_all(format!("{}/.{name}/credentials", path.display())).unwrap();
                println!("created {}/.{name}/credentials", &path.display());
            }

            println!("{}{registry_link}", MESSAGES.get("registry_login").unwrap());

            let identity_string: String;
            let password_string: String;
            let client = reqwest::blocking::Client::new();
            let identity = Text::new("identity:").prompt();
            let password = Password::new("password:")
                .with_display_toggle_enabled()
                .with_display_mode(PasswordDisplayMode::Masked)
                .with_validator(min_length!(8))
                .without_confirmation()
                .prompt();

            match identity {
                Ok(value) => identity_string = value.clone(),
                Err(_) => std::process::exit(1),
            };

            match password {
                Ok(value) => password_string = value.clone(),
                Err(_) => std::process::exit(1),
            };

            let pb = ProgressBar::new_spinner();
            pb.enable_steady_tick(std::time::Duration::from_millis(80));
            pb.set_style(ProgressStyle::with_template("{spinner:.yellow} {msg}").unwrap().tick_strings(&[
                "[    ]", "[=   ]", "[==  ]", "[=== ]", "[ ===]", "[  ==]", "[   =]", "[    ]", "[   =]", "[  ==]", "[ ===]", "[====]", "[=== ]", "[==  ]", "[=   ]", "",
            ]));
            pb.set_message("logging in...");

            let response = client
                .post(format!("{registry_link}/api/collections/just_auth_system/auth-with-password"))
                .body(format!("{{\"identity\":\"{identity_string}\",\"password\":\"{password_string}\"}}"))
                .header(reqwest::header::CONTENT_TYPE, reqwest::header::HeaderValue::from_static("application/json"))
                .send();

            match response {
                Ok(response) => {
                    match serde_json::from_str::<Response>(&response.text().unwrap()) {
                        Ok(json) => {
                            let mut file = std::fs::File::create(format!("{}/.{name}/credentials/{}].json", path.display(), registry_link.replace("://", "["))).unwrap();
                            file.write_all(format!("{{\"token\":\"{}\",\"access\":\"{}\"}}", json.token, json.record.id).as_bytes()).unwrap();
                            pb.finish_with_message(format!(
                                "\x08{} {} {}",
                                "✔".green(),
                                MESSAGES.get("login_msg").unwrap().bright_green(),
                                format!("({})", json.record.id).white()
                            ));
                        }
                        Err(_) => {
                            eprint!("\r{} {}\n", "✖".red(), MESSAGES.get("login_error").unwrap().bright_red());
                            std::process::exit(1);
                        }
                    };
                }
                Err(err) => eprint!("\r{} {}\n", "✖".red(), format!("{}{}", MESSAGES.get("login_error_generic").unwrap(), err.to_string()).bright_red()),
            };
        }
        None => {
            eprintln!("{}", MESSAGES.get("home_error").unwrap().red());
            std::process::exit(1);
        }
    }
}

pub fn logout() {
    let name = global!("vendor.name");

    match home::home_dir() {
        Some(path) => {
            if let Err(_) = std::fs::remove_file(format!("{}/.{name}/credentials.json", path.display())) {
                eprintln!("{}", MESSAGES.get("logout_error").unwrap().red());
            } else {
                println!("{}", MESSAGES.get("logout_msg").unwrap().green())
            }
        }
        None => {
            eprintln!("{}", MESSAGES.get("home_error").unwrap().red());
            std::process::exit(1);
        }
    }
}
