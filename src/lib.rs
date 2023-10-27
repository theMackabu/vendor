#[macro_use]
extern crate lazy_static;

use global_placeholders::init;
use std::collections::HashMap;

lazy_static! {
    pub static ref MESSAGES: HashMap<&'static str, &'static str> = {
        let mut default = HashMap::new();
        default.insert("template_error", "unable create template, please try again");
        default.insert("template_downloaded", "downloaded template");
        default.insert("create_error_1", "An error happened when asking for ");
        default.insert("create_error_2", ", try again later.");
        default.insert("create_project", "This utility will walk you through creating a package.yml file.\n");
        default.insert("saved_project", "\n✨ success, saved package.yml");
        default.insert("read_error", "unable to find package.yml");
        default.insert("yaml_error", " in package.yml");
        default.insert("registry_login", "logging into ");
        default.insert("home_error", "Impossible to get your home dir.");
        default.insert("login_error_generic", "unable to login: ");
        default.insert("login_error", "unable to login, invalid username or password");
        default.insert("logout_error", "unable to logout, no token file (are you logged in?)");
        default.insert("login_msg", "logged in");
        default.insert("logout_msg", "logged out");
        default.insert("tar_error", "- unable to remove temporary tarfile. does it exist?");
        default.insert("cwd_error", "cannot retrive current directory");
        default.insert("unpack_error", "failed to unpack tarball");
        default.insert("publish_error", "unable to publish package");
        default.insert("publish_error_generic", "unable to publish package, please try again");
        default.insert("publish_error_login", "unable to publish package, please login");
        default.insert("publish_error_token", "your token might be expired, please login again");
        default.insert("publish_msg", "publishing");
        default.insert("publish_wait", "publishing...");
        default.insert("publish_done", "created package ");
        default.insert("file_error", "unable remove file, please try again");
        default.insert("file_error", "unable remove file, please try again");
        default.insert("find_error", "unable to find ");
        default.insert("remove_error", "unable to remove ");
        default.insert("pkg_error", "unable to add package ");
        default.insert("pkg_skip", "xskipped installed package ");
        default.insert("pkg_fs_error", "unable to add package, filesystem error");
        default.insert("pkg_create_error", "Failed to create file ");
        default.insert("pkg_add_error", "unable to add ");
        default.insert("pkg_fetch_error", "failed to get from ");
        default.insert("pkg_content_error", "failed to get content length of ");
        default.insert("pkg_find_msg", "locating...");
        default.insert("pkg_found", "located package ");
        default.insert("dep_found", "located dependency ");
        default.insert("dep_skip", "skipped installed dependency ");
        default.insert("write_error", "Error while downloading file");
        default.insert("download_error", "Error while writing to file");
        default.insert("install_done", "✨ done in ");
        default.insert("is_question_install", "is it installed?");
        default.insert("pkg_unused", "removed unused package ");
        default.insert("pkg_remove", "removed package ");
        default.insert("clean_error", "unable to clean packages, try again");

        return default;
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
