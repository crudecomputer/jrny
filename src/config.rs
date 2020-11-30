use serde::Deserialize;
use std::fs;

use crate::paths::ProjectPaths;

pub struct Config {
    pub paths: ProjectPaths,
    pub settings: Settings,
}

impl Config {
    pub fn new(conf_path_name: Option<&str>) -> Result<Self, String> {
        let paths = ProjectPaths::from_conf_path(conf_path_name)?;

        let contents = fs::read_to_string(&paths.conf)
            .expect(&format!("Could not open {}", paths.conf.display()));

        let settings: Settings = toml::from_str(&contents)
            .expect(&format!("Could not parse {}", paths.conf.display()));

        Ok(Self { paths, settings })
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Settings {
    pub connection: ConnectionSettings,
    pub table: TableSettings,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ConnectionSettings {
    pub host: String,
    pub name: String,
    pub port: u16,
    pub user: String,
    //pub pass: Option<String>
}

#[derive(Clone, Debug, Deserialize)]
pub struct TableSettings {
    pub schema: String,
    pub name: String,
}
