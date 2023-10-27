use crate::MESSAGES;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;

#[derive(Debug, Deserialize, Serialize)]
pub struct Info {
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub url: String,
    pub repository: String,
    pub license: String,
    pub index: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Registry {
    pub public: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Package {
    pub info: Info,
    pub registry: Registry,
    pub dependencies: BTreeMap<String, String>,
}

pub fn read() -> Package {
    let contents = match fs::read_to_string("package.yml") {
        Ok(content) => content,
        Err(_) => {
            eprintln!("{} {}", "✖".red(), MESSAGES.get("read_error").unwrap().bright_red());
            std::process::exit(1);
        }
    };

    let yaml_file: Result<Package, _> = serde_yaml::from_str(&contents);
    let parsed = match yaml_file {
        Ok(project) => project,
        Err(error) => {
            eprintln!("{}", format!("{}{}", MESSAGES.get("yaml_error").unwrap(), error).red());
            std::process::exit(1);
        }
    };

    return parsed;
}
