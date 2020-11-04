use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

use crate::{CONF, PathWithName};

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub app: AppConfig,
    pub connection: ConnectionConfig,
}

impl Config {
    pub fn new(conf_path_name: Option<&str>) -> Result<Self, String> {
        let conf = PathWithName::new("config file", PathBuf::from(
            if let Some(name) = conf_path_name { name } else { CONF }
        ))?;

        let contents = fs::read_to_string(conf.path)
            .expect(&format!("Could not open {}", conf.name));

        Ok(toml::from_str(&contents).expect(&format!("Could not parse {}", CONF)))
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct AppConfig {
    pub schema: String,
    pub table: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ConnectionConfig {
    pub host: String,
    pub name: String,
    pub port: u16,
    pub user: String,
    //pub pass: Option<String>
}
