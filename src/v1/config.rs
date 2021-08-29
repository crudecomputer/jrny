use std::{env, fs};

use serde::Deserialize;

use crate::{
    paths::ProjectPaths,
    revisions::RevisionFile,
    Error,
    Result,
};

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

#[derive(Debug)]
pub struct Config {
    pub paths: ProjectPaths,
    pub settings: Settings,
    pub next_id: i32,
}

impl Config {
    pub fn new(conf_path_name: Option<&str>) -> Result<Self> {
        let paths = ProjectPaths::from_conf_path(conf_path_name)?;

        // TODO This is unwieldy now...
        if !paths.conf.exists() {
            return Err(Error::ConfigNotFound(paths.conf.display().to_string()));
        }

        if !paths.conf.is_file() {
            return Err(Error::FileNotValid(paths.conf.display().to_string()));
        }

        if paths.env.exists() && !paths.env.is_file() {
            return Err(Error::FileNotValid(paths.env.display().to_string()));
        }

        let contents = fs::read_to_string(&paths.conf)?;
        let toml_settings: TomlSettings = toml::from_str(&contents)
            .map_err(|e| Error::ConfigInvalid(e, paths.conf.display().to_string()))?;

        let settings = Settings {
            connection: ConnectionSettings {
                database_url: url_from_toml(&toml_settings.connection)?,
            },
            table: toml_settings.table,
        };

        let next_id = RevisionFile::all_from_disk(&paths.revisions)?
            .iter()
            .reduce(|rf1, rf2| if rf1.id > rf2.id { rf1 } else { rf2 })
            .map_or(0, |rf| rf.id as i32)
            + 1;

        let config = Self {
            paths,
            settings,
            next_id,
        };

        Ok(config)
    }
}

fn url_from_toml(conn_settings: &TomlConnectionSettings) -> Result<String> {
    Ok(match &conn_settings.strategy {
        ConnectionStrategy::EnvUrlString { var_name } => {
            env::var(var_name).map_err(|e| Error::BadEnvVar(e, var_name.clone()))?
        }
    })
}
