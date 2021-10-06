use std::{env, fs, path::PathBuf};

use serde::Deserialize;

use crate::{
    error::Error,
    Result,
};

#[derive(Clone, Debug, Deserialize)]
pub struct DatabaseEnvironment {
    pub url: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ProjectEnvironment {
    pub database: DatabaseEnvironment,
}

impl ProjectEnvironment {
    pub fn new(envpath: &PathBuf, database_url: Option<String>) -> Result<Self> {
        // Should the file be checked if overridden by `database_url` and not used?
        if envpath.exists() && !envpath.is_file() {
            return Err(Error::FileNotValid(envpath.display().to_string()));
        }

        match database_url {
            Some(url) => Ok(Self {
                database: DatabaseEnvironment { url: url.to_owned() }
            }),
            None => {
                if !envpath.exists() {
                    return Err(Error::EnvNotFound);
                }
                let contents = fs::read_to_string(&envpath)?;
                let env: Result<Self> = toml::from_str(&contents)
                    .map_err(|e| Error::TomlInvalid(e, envpath.display().to_string()));

                env
            }
        }
    }
}
