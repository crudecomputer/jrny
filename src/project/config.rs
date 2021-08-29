
use std::{env, fs};

use serde::Deserialize;

use crate::{
    paths::ProjectPaths,
    revisions::RevisionFile,
    Error,
    Result,
};

#[derive(Debug)]
pub struct ProjectConfig {
    pub settings: Settings,
    pub next_id: i32,
}

#[derive(Clone, Debug, Deserialize)]
struct Settings {
    pub table: TableSettings,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TableSettings {
    pub schema: String,
    pub name: String,
}

impl Config {
    pub fn new(conf_path_name: Option<&str>) -> Result<Self> {
        let paths = ProjectPaths::from_conf_path(conf_path_name)?;

        if !paths.conf.exists() {
            return Err(Error::ConfigNotFound(paths.conf.display().to_string()));
        }

        if !paths.conf.is_file() {
            return Err(Error::FileNotValid(paths.conf.display().to_string()));
        }

        let contents = fs::read_to_string(&paths.conf)?;
        let settings: Settings = toml::from_str(&contents)
            .map_err(|e| Error::ConfigInvalid(e, paths.conf.display().to_string()))?;

        let next_id = RevisionFile::all_from_disk(&paths.revisions)?
            .iter()
            .reduce(|rf1, rf2| if rf1.id > rf2.id { rf1 } else { rf2 })
            .map_or(0, |rf| rf.id as i32)
            + 1;

        let config = Self {
            settings,
            next_id,
        };

        Ok(config)
    }
}
