use crate::{project, MESSAGES};

use anyhow::Context;
use brown;
use colored::Colorize;
use flate2::read::GzDecoder;
use futures_util::StreamExt;
use global_placeholders::global;
use indicatif::{HumanDuration, ProgressBar, ProgressStyle};
use macros_rs::{fmtstr, string, ternary};
use std::cmp::min;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::{fs, fs::File, io::Write};
use tar::Archive;

#[derive(Debug, serde::Deserialize)]
struct Dist {
    version: String,
    tarball: String,
}

#[derive(Debug, serde::Deserialize)]
struct Response {
    dist: Dist,
}

fn remove_file(file: &str) {
    if let Err(_) = fs::remove_file(file) {
        eprintln!("{} {}", "✖".red(), MESSAGES.get("file_error").unwrap().bright_red());
        std::process::exit(1);
    }
}
fn move_package(file: &str, name: &str, version: &str) {
    let current_dir = std::env::current_dir().expect(MESSAGES.get("cwd_error").unwrap());
    let mut package = project::package::read();
    let dependencies = package.dependencies.clone();

    if !std::path::Path::new(fmtstr!("{}/packages", current_dir.display())).is_dir() {
        std::fs::create_dir_all(format!("{}/packages", current_dir.display())).unwrap();
        log::debug!("created {}/packages", current_dir.display())
    }

    match File::open(file) {
        Ok(tarball) => {
            let tar = GzDecoder::new(tarball);
            let mut archive = Archive::new(tar);

            archive
                .unpack(format!("{}/packages/{name}@{version}", current_dir.display()))
                .expect(MESSAGES.get("unpack_error").unwrap());
            remove_file(file);

            if package.dependencies.get(name) == None {
                package.dependencies.insert(name.to_string(), version.to_string());
            } else {
                let mut versions = dependencies.get(name).unwrap().split(",").collect::<Vec<&str>>();

                if versions.last().unwrap() != &version {
                    versions.push(&version);
                    package.dependencies.insert(name.to_string(), versions.join(",").trim_matches(' ').to_string());
                }
            }

            if let Err(err) = File::create("package.yml").unwrap().write_all(serde_yaml::to_string(&package).unwrap().as_bytes()) {
                eprintln!("{} {}", "✖".red(), format!("{}{name}@{version}, {err}", MESSAGES.get("pkg_add_error").unwrap()).bright_red());
                std::process::exit(1);
            };
        }
        Err(_) => {
            eprintln!("{} {}", "✖".red(), MESSAGES.get("pkg_fs_error").unwrap().bright_red());
            remove_file(file);
            std::process::exit(1);
        }
    }
}

pub async fn download(client: &reqwest::Client, url: &str, path: &str, package_info: String) -> Result<(), String> {
    let res = client
        .get(url)
        .send()
        .await
        .or(Err(format!("\r{} {}\n", "✖".red(), format!("{}{}", MESSAGES.get("pkg_fetch_error").unwrap(), &url).bright_red())))?;

    let total_size = res
        .content_length()
        .ok_or(format!("\r{} {}\n", "✖".red(), format!("{}{}", MESSAGES.get("pkg_content_error").unwrap(), &url).bright_red()))?;

    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::with_template("{msg}: [{bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})").unwrap());
    pb.set_message(format!("{}", format!("+ {package_info}").bright_cyan()));

    let mut file = File::create(path).or(Err(format!("{}'{}'", MESSAGES.get("pkg_create_error").unwrap(), path)))?;
    let mut downloaded: u64 = 0;
    let mut stream = res.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item.or(Err(string!(MESSAGES.get("download_error").unwrap())))?;
        file.write_all(&chunk).or(Err(string!(MESSAGES.get("write_error").unwrap())))?;
        let new = min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
        pb.set_position(new);
    }

    pb.finish_with_message(format!("{}", format!("+ {package_info}").bright_cyan()));
    return Ok(());
}

pub fn install() {
    let started = Instant::now();
    let packages = project::package::read().dependencies;
    for (name, versions) in &packages {
        for ver in versions.split(",").collect::<Vec<&str>>() {
            add(&format!("{}@{}", name, ver.trim_matches(' ')), false)
        }
    }
    println!("{}", format!("{}{}", MESSAGES.get("install_done").unwrap(), HumanDuration(started.elapsed())).yellow());
}

pub fn add(input: &str, timer: bool) {
    let app_name = global!("vendor.name");
    let registry = global!("vendor.registry");

    let version;
    let started = Instant::now();
    let name = input.split("@").collect::<Vec<&str>>()[0];
    let current_dir = std::env::current_dir().expect(MESSAGES.get("cwd_error").unwrap());
    let client = reqwest::blocking::Client::builder().user_agent(format!("{app_name}/{}", env!("CARGO_PKG_VERSION"))).build().unwrap();
    let style = ProgressStyle::with_template("{spinner:.yellow} {msg}").unwrap().tick_strings(&[
        "[    ]", "[=   ]", "[==  ]", "[=== ]", "[ ===]", "[  ==]", "[   =]", "[    ]", "[   =]", "[  ==]", "[ ===]", "[====]", "[=== ]", "[==  ]", "[=   ]", "",
    ]);

    let package_info = ternary!(
        input.split("@").collect::<Vec<&str>>().len() > 1,
        format!("{}@{}", input.split("@").collect::<Vec<&str>>()[0], input.split("@").collect::<Vec<&str>>()[1]),
        input.split("@").collect::<Vec<&str>>()[0].to_string()
    );

    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(Duration::from_millis(80));
    pb.set_style(style.clone());
    pb.set_message(string!(MESSAGES.get("pkg_find_msg").unwrap()));

    match client.get(format!("{registry}/{package_info}")).send() {
        Ok(res) => {
            match serde_json::from_str::<Response>(&res.text().unwrap()) {
                Ok(json) => {
                    version = json.dist.version.clone();
                    if !std::path::Path::new(fmtstr!("{}/packages/{name}@{version}", current_dir.display())).is_dir() {
                        pb.finish_with_message(format!("\x08{} {}", "✔".green(), format!("{}{name}@{}", MESSAGES.get("pkg_found").unwrap(), json.dist.version).green()));

                        let runtime = tokio::runtime::Runtime::new().unwrap();
                        match runtime.block_on(download(&reqwest::Client::new(), &json.dist.tarball, &format!("{name}.tgz"), format!("{name}@{}", &json.dist.version))) {
                            Ok(_) => move_package(&format!("{name}.tgz"), &name, &json.dist.version),
                            Err(err) => {
                                eprint!(
                                    "\r{} {}\n",
                                    "✖".red(),
                                    format!("{}{}: {}", MESSAGES.get("pkg_error").unwrap(), package_info, err.to_string()).bright_red()
                                );
                                std::process::exit(1);
                            }
                        };
                    } else {
                        pb.finish_with_message(format!(
                            "\x08{} {}",
                            "ℹ".magenta(),
                            format!("{}{name}@{}", MESSAGES.get("pkg_skip").unwrap(), json.dist.version).bright_magenta()
                        ));
                    }
                }
                Err(_) => {
                    pb.finish_with_message(format!("\x08{} {}", "✖".red(), format!("{}{}", MESSAGES.get("find_error").unwrap(), package_info).bright_red()));
                    std::process::exit(1);
                }
            };
        }
        Err(err) => {
            eprint!(
                "\r{} {}\n",
                "✖".red(),
                format!("{}{}: {}", MESSAGES.get("pkg_error").unwrap(), package_info, err.to_string()).bright_red()
            );
            std::process::exit(1);
        }
    };

    match reqwest::blocking::get(format!(
        "{registry}/api/v{}/dependencies/{}",
        env!("CARGO_PKG_VERSION").split(".").collect::<Vec<&str>>().join(""),
        input.split("@").collect::<Vec<&str>>()[0].to_string()
    )) {
        Ok(res) => {
            match serde_json::from_str::<HashMap<String, Vec<String>>>(&res.text().unwrap()) {
                Ok(json) => {
                    for link in &json[&version] {
                        let pb_dep = ProgressBar::new_spinner();
                        let name = link.split("/").collect::<Vec<&str>>()[3];
                        let version = link.split("/").collect::<Vec<&str>>()[5];

                        pb_dep.enable_steady_tick(Duration::from_millis(80));
                        pb_dep.set_style(style.clone());
                        pb_dep.set_message("locating...");

                        if !std::path::Path::new(fmtstr!("{}/packages/{name}@{version}", current_dir.display())).is_dir() {
                            pb_dep.finish_with_message(format!("\x08{} {}", "✔".green(), format!("{}{name}@{}", MESSAGES.get("dep_found").unwrap(), &version).bright_green()));
                        } else {
                            pb_dep.finish_with_message(format!(
                                "\x08{} {}",
                                "ℹ".magenta(),
                                format!("{}{name}@{}", MESSAGES.get("dep_skip").unwrap(), &version).bright_magenta()
                            ));
                        }
                    }

                    for link in &json[&version] {
                        let name = link.split("/").collect::<Vec<&str>>()[3];
                        let version = link.split("/").collect::<Vec<&str>>()[5];
                        let runtime = tokio::runtime::Runtime::new().unwrap();
                        if !std::path::Path::new(fmtstr!("{}/packages/{name}@{version}", current_dir.display())).is_dir() {
                            match runtime.block_on(download(&reqwest::Client::new(), link, &format!("{name}.tgz"), format!("{name}@{}", version))) {
                                Ok(_) => move_package(&format!("{name}.tgz"), &name, &version),
                                Err(err) => {
                                    eprint!(
                                        "\r{} {}\n",
                                        "✖".red(),
                                        format!("{}{}: {}", MESSAGES.get("pkg_error").unwrap(), package_info, err.to_string()).bright_red()
                                    );
                                    std::process::exit(1);
                                }
                            };
                        }
                    }
                }
                Err(_) => {}
            };
        }
        Err(err) => {
            eprint!(
                "\r{} {}\n",
                "✖".red(),
                format!("{}{}: {}", MESSAGES.get("pkg_error").unwrap(), package_info, err.to_string()).bright_red()
            );
            std::process::exit(1);
        }
    };
    if timer {
        println!("{}", format!("{}{}", MESSAGES.get("install_done").unwrap(), HumanDuration(started.elapsed())).yellow());
    }
}

pub fn remove(name: &String) {
    let started = Instant::now();
    let current_dir = std::env::current_dir().expect(MESSAGES.get("cwd_error").unwrap());
    let mut package = project::package::read();
    let dependencies = package.dependencies.clone();
    let key = name.split("@").collect::<Vec<&str>>()[0];
    let generic_error = |err: String| -> String { format!("{} {}", "✖".red(), format!("{}{name}, {err}", MESSAGES.get("remove_error").unwrap()).bright_red()) };

    let mut versions = match dependencies.get(key).with_context(|| generic_error(string!(MESSAGES.get("is_question_install").unwrap()))) {
        Ok(content) => content.split(",").collect::<Vec<&str>>(),
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1);
        }
    };

    let package_dir = ternary!(
        name.split("@").collect::<Vec<&str>>().len() > 1,
        format!("{}/{}", name.split("@").collect::<Vec<&str>>()[0], name.split("@").collect::<Vec<&str>>()[1]),
        name.split("@").collect::<Vec<&str>>()[0].to_string()
    );

    if let Err(_) = std::fs::remove_dir_all(format!("{}/packages/{package_dir}", current_dir.display())) {
        eprintln!("{}", generic_error(string!(MESSAGES.get("is_question_install").unwrap())));
        std::process::exit(1);
    } else {
        if name.split("@").collect::<Vec<&str>>().len() > 1 {
            if versions.len() > 1 {
                versions.remove(versions.iter().position(|x| &*x.trim_matches(' ') == name.split("@").collect::<Vec<&str>>()[1]).unwrap());
                package.dependencies.remove(key);
                package.dependencies.insert(key.to_string(), String::from(versions.join(",").trim_matches(' ')));
            } else {
                package.dependencies.remove(key);
            }
        } else {
            package.dependencies.remove(name);
        }

        if let Err(err) = File::create("package.yml").unwrap().write_all(serde_yaml::to_string(&package).unwrap().as_bytes()) {
            eprintln!("{}", generic_error(err.to_string()));
            std::process::exit(1);
        }
        println!("\x08{} {}", "✔".green(), format!("{}{name}", MESSAGES.get("pkg_remove").unwrap()).green());
    }

    println!("{}", format!("{}{}", MESSAGES.get("install_done").unwrap(), HumanDuration(started.elapsed())).yellow());
}

pub fn clean() {
    let package = project::package::read();
    let dependencies = package.dependencies.clone();
    let generic_error = |name: &str, err: &str| -> String { format!("{} {}", "✖".red(), format!("{}{name}, {err}", MESSAGES.get("remove_error").unwrap()).bright_red()) };

    match brown::get_dirs("packages") {
        Ok(paths) => {
            for path in paths {
                let package_dir = brown::direntry_to_path(&path).unwrap();
                let package_name = package_dir.split('/').last().unwrap();

                if dependencies.get(package_name).is_none() {
                    if let Err(_) = brown::remove_dir_brute(&package_dir) {
                        eprintln!("{}", generic_error(package_name, MESSAGES.get("is_question_install").unwrap()));
                        std::process::exit(1);
                    } else {
                        println!("\x08{} {}", "✔".blue(), format!("{}{package_name}", MESSAGES.get("pkg_unused").unwrap()).bright_blue());
                    }
                }
            }
        }
        Err(_) => eprintln!("{} {}", "✖".red(), MESSAGES.get("clean_error").unwrap().bright_red()),
    };
}
