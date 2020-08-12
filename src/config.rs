use std::io::prelude::*;
use std::fs::File;
use std::path::Path;

use serde::{Deserialize, Serialize};


#[allow(dead_code)]
const FILE_NAME: &'static str = "chattery.toml";


#[derive(Deserialize, Serialize)]
pub struct Config {
    ip: String,
    port: u16,
    max_clients: usize,
    max_acceptable_buffer: usize,
}

impl Config {
    pub fn init() -> Config {
        let config_file = Path::new(FILE_NAME);
        let fallback = Config::default();
        
        if !config_file.is_file() {
            let fallback_content = toml::to_string(&fallback)
                .expect("Could not initialize new config file from default fallback!");

            let mut file = File::create(config_file)
                .expect(&format!("Could not open file {}!", FILE_NAME));

            file.write(fallback_content.as_bytes()).ok();
            return fallback;
        }

        let mut buffer = String::new();
        File::open(config_file)
            .expect(&format!("Could not open file {}!", FILE_NAME))
            .read_to_string(&mut buffer)
            .expect("Could not read the configuraton file!");

        toml::from_str::<Config>(&buffer)
            .expect("Could not initialize config from existed file!")
    }

    pub fn get_address(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }

    pub fn get_max_clients(&self) -> usize {
        self.max_clients
    }

    pub fn get_acceptable_buffer(&self) -> usize {
        self.max_acceptable_buffer
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ip: String::from("127.0.0.1"),
            port: 2424,
            max_clients: 4,
            max_acceptable_buffer: 256,
        }
    }
}