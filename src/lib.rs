pub mod netease_music;

use std::path::PathBuf;

use li_logger::get_logger;
use serde::{Deserialize, Serialize};

lazy_static::lazy_static! {
    pub static ref CONFIG: Config = Config::load();
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub cookie: String
}

impl Default for Config {
    fn default() -> Self {
        Config {
            host: "127.0.0.1".to_string(),
            port: 8037,
            cookie: "".to_string()
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let config_path = PathBuf::from("config.json");
        if !config_path.exists() {
            let config = Config::default();
            let config_file = std::fs::File::create("config.json").unwrap();
            serde_json::to_writer_pretty(config_file, &config).unwrap();
            get_logger().strong().warn("A new config file is created! Please edit it with a cookie.");
            config
        } else {
            let config_file = std::fs::File::open("config.json").unwrap();
            serde_json::from_reader(config_file).unwrap()
        }
    }
}