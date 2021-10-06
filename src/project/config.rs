use std::{env, fs, path::PathBuf};

use serde::Deserialize;

use crate::{
    Error,
    Result,
};

#[derive(Clone, Debug, Deserialize)]
pub struct TableSettings {
    pub schema: String,
    pub name: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ProjectConfig {
    pub table: TableSettings,
}

impl ProjectConfig {
    pub fn new(confpath: &PathBuf) -> Result<Self> {
        if !confpath.exists() {
            return Err(Error::ConfigNotFound(confpath.display().to_string()));
        }

        if !confpath.is_file() {
            return Err(Error::FileNotValid(confpath.display().to_string()));
        }

        let contents = fs::read_to_string(&confpath)?;
        let config: Self = toml::from_str(&contents)
            .map_err(|e| Error::TomlInvalid(e, confpath.display().to_string()))?;

        Ok(config)
    }
}
