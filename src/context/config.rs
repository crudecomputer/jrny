use std::fs;
use std::path::PathBuf;

use serde::Deserialize;

use crate::{Error, Result};

/// Configuration values specific to the revision files on disk.
#[derive(Clone, Debug, Deserialize)]
pub struct RevisionsSettings {
    /// The directory containing the SQL revision files
    pub directory: PathBuf,
}

/// Configuration values indicating the database table in which to store revision metadata.
#[derive(Clone, Debug, Deserialize)]
pub struct TableSettings {
    /// The name of the database schema containing the revisions table
    pub schema: String,
    /// The name of the table to hold revisions metadata
    pub name: String,
}

/// Project-specific settings that do not contain sensitive information and
/// are likely to be consistent across environments.
#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub revisions: RevisionsSettings,
    pub table: TableSettings,
}

impl Config {
    /// Attempts to load a TOML file from the given path and serialize it
    /// into a `Config` instance.
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
