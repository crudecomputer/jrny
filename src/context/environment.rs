use std::fs;
use std::path::PathBuf;

use serde::Deserialize;

use crate::{Error, Result};

#[derive(Clone, Debug, Deserialize)]
pub struct DatabaseEnvironment {
    pub url: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Environment {
    pub database: DatabaseEnvironment,
}

impl Environment {
    pub fn from_filepath(envpath: &PathBuf) -> Result<Self> {
        if !envpath.exists() {
            return Err(Error::EnvNotFound);
        }

        if envpath.exists() && !envpath.is_file() {
            return Err(Error::FileNotValid(envpath.display().to_string()));
        }

        let contents = fs::read_to_string(envpath)?;
        let env: Result<Self> = toml::from_str(&contents)
            .map_err(|e| Error::TomlInvalid(e, envpath.display().to_string()));

        env
    }

    pub fn from_database_url(url: &str) -> Self {
        Self {
            database: DatabaseEnvironment {
                url: url.to_owned(),
            },
        }
    }
}
