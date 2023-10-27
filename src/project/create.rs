use colored::Colorize;
use std::io::Cursor;
use std::path::PathBuf;
use zip_extract::extract;

pub fn create_template(name: &str, registry: &String) {
    let target_dir = PathBuf::from(name);
    match reqwest::blocking::get(format!("{registry}/api/{}/templates/{name}.zip", env!("CARGO_PKG_VERSION").split(".").collect::<Vec<&str>>().join(""))) {
        Ok(res) => {
            if let Err(_) = extract(Cursor::new(&res.bytes().unwrap()), &target_dir, true) {
                eprintln!("{} {}", "✖".red(), "unable create template, please try again".bright_red());
            } else {
                println!("\x08{} {}", "✔".green(), format!("downloaded template {name}").green());
            }
        }
        Err(_) => {
            eprintln!("{} {}", "✖".red(), "unable create template, please try again".bright_red());
        }
    };
}
