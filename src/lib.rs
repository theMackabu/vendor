#[macro_use]
extern crate lazy_static;

use global_placeholders::init;
use std::collections::HashMap;

lazy_static! {
    pub static ref MESSAGES: HashMap<&'static str, &'static str> = {
        let mut default = HashMap::new();
        default.insert("template_error", "unable create template, please try again");
        default.insert("template_downloaded", "downloaded template");
        default
    };
}

pub fn set_registry(link: &str) {
    init!("vendor.registry", link);
}

pub fn set_name(name: &str) {
    init!("vendor.name", name);
}

pub fn set_messages(mut msg: HashMap<&str, &str>) {
    for (key, val) in MESSAGES.iter() {
        if !msg.contains_key(key) {
            msg.insert(key, val);
        }
    }
}

pub mod helpers;
pub mod project;
pub mod registry;
