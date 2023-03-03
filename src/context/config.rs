use std::fs;
use std::path::PathBuf;

use serde::Deserialize;

use crate::{Error, Result};

#[derive(Clone, Debug, Deserialize)]
pub struct RevisionsSettings {
    pub directory: PathBuf,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TableSettings {
    pub schema: String,
    pub name: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub revisions: RevisionsSettings,
    pub table: TableSettings,
}

impl Config {
    pub fn from_filepath(confpath: &PathBuf) -> Result<Self> {
        if !confpath.exists() {
            return Err(Error::ConfigNotFound(confpath.display().to_string()));
        }

        if !confpath.is_file() {
            return Err(Error::FileNotValid(confpath.display().to_string()));
        }

        let contents = fs::read_to_string(confpath)?;
        let mut config: Self = toml::from_str(&contents)
            .map_err(|e| Error::TomlInvalid(e, confpath.display().to_string()))?;

        // The revisions directory is relative to the config file itself,
        // not the current working directory.
        config.revisions.directory = confpath.parent().unwrap().join(&config.revisions.directory);

        Ok(config)
    }
}
