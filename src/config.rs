use serde::Deserialize;
use std::{env, fs};

use crate::paths::ProjectPaths;

/// Strategy for locating connection details.
/// Currently only supports whole URL-style string but it could be extended to support
/// loading config files, using separate ENV vars for host, etc.
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "kebab-case")]
enum ConnectionStrategy {

    #[serde(rename_all = "kebab-case")]
    EnvUrlString { var_name: String },
}

#[derive(Clone, Debug)]
pub struct Settings {
    pub connection: ConnectionSettings,
    pub table: TableSettings,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ConnectionSettings {
    pub database_url: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TableSettings {
    pub schema: String,
    pub name: String,
}

#[derive(Clone, Debug, Deserialize)]
struct TomlConnectionSettings {
    pub strategy: ConnectionStrategy,
}

#[derive(Clone, Debug, Deserialize)]
struct TomlSettings {
    pub connection: TomlConnectionSettings,
    pub table: TableSettings,
}

pub struct Config {
    pub paths: ProjectPaths,
    pub settings: Settings,
}

impl Config {
    pub fn new(conf_path_name: Option<&str>) -> Result<Self, String> {
        let paths = ProjectPaths::from_conf_path(conf_path_name)?;

        let contents = fs::read_to_string(&paths.conf)
            .unwrap_or_else(|e| panic!("Could not open {}: {}", paths.conf.display(), e.to_string()));

        let toml_settings: TomlSettings = toml::from_str(&contents)
            .unwrap_or_else(|e| panic!("Could not open {}: {}", paths.conf.display(), e.to_string()));

        let settings = Settings {
            connection: ConnectionSettings {
                database_url: url_from_toml(&toml_settings.connection)?,
            },
            table: toml_settings.table,
        };

        Ok(Self { paths, settings })
    }
}

fn url_from_toml(conn_settings: &TomlConnectionSettings) -> Result<String, String> {
    Ok(match &conn_settings.strategy {
        ConnectionStrategy::EnvUrlString { var_name } =>
            env::var(var_name).map_err(|e| format!("{}: {}", e, var_name))?,
    })
}
