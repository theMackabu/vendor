use crate::MESSAGES;

use colored::Colorize;
use global_placeholders::global;
use std::io::Cursor;
use std::path::PathBuf;
use zip_extract::extract;

pub fn create_template(name: &str) {
    let target_dir = PathBuf::from(name);
    match reqwest::blocking::get(format!(
        "{}/api/{}/templates/{name}.zip",
        global!("vendor.registry"),
        env!("CARGO_PKG_VERSION").split(".").collect::<Vec<&str>>().join("")
    )) {
        Ok(res) => {
            if let Err(_) = extract(Cursor::new(&res.bytes().unwrap()), &target_dir, true) {
                eprintln!("{} {}", "✖".red(), MESSAGES.get("template_error").unwrap().bright_red());
            } else {
                println!("\x08{} {}", "✔".green(), format!("{} {name}", MESSAGES.get("template_downloaded").unwrap()).green());
            }
        }
        Err(_) => {
            eprintln!("{} {}", "✖".red(), MESSAGES.get("template_error").unwrap().bright_red());
        }
    };
}
