
//! Utilities for working with paths.
use std::path::PathBuf;

use crate::{Error, Result, CONF, ENV, ENV_EX};

/// A container for the various paths of interest for a project.
#[derive(Debug)]
pub struct ProjectPaths {
    pub conf_file: PathBuf,
    pub env_file: PathBuf,
    pub env_ex_file: PathBuf,
    pub revisions_dir: PathBuf,
    pub root_dir: PathBuf,
}

impl ProjectPaths {
    /// Creates path bufs for a new project given a root directory, ensuring that
    /// there is not already a config & env file or a non-empty revisions directory.
    pub fn for_new_project(root_dir: &PathBuf) -> Result<Self> {
        let root_dir = root_dir.clone();

        let paths = Self {
            conf_file: root_dir.join(CONF),
            env_file: root_dir.join(ENV),
            env_ex_file: root_dir.join(ENV_EX),
            revisions_dir: root_dir.join("revisions"),
            root_dir,
        };

        paths.valid_for_new()?;

        Ok(paths)
    }

    /// Creates path bufs for a project either relative to the given config filepath name
    /// or to the current working directory if no path name is provided.
    pub fn from_conf_path(conf_path_name: Option<&str>) -> Result<Self> {
        let conf_file = PathBuf::from(conf_path_name.unwrap_or(CONF));

        let root_dir = conf_file
            .parent()
            .ok_or_else(|| Error::PathInvalid(conf_file.display().to_string()))?
            .to_path_buf();

        Ok(Self {
            conf_file,
            env_file: root_dir.join(ENV),
            env_ex_file: root_dir.join(ENV_EX),
            revisions_dir: root_dir.join("revisions"),
            root_dir,
        })
    }

    /// Ensures that own path bufs are valid for a new project, namely that the
    /// root path is a directory if exists, that the revisions directory is empty
    /// if exists, and that no config or environment files exists
    fn valid_for_new(&self) -> Result<()> {
        use Error::*;

        // TODO Returning MULTIPLE errors might actually be nicer for the user
        // rather than whichever fails first

        if self.root_dir.exists() && !self.root_dir.is_dir() {
            return Err(PathNotDirectory(self.root_dir.display().to_string()));
        }

        if self.revisions_dir.exists() && !is_empty_dir(&self.revisions_dir)? {
            return Err(PathNotEmptyDirectory(self.revisions_dir.display().to_string()));
        }

        for f in [&self.conf_file, &self.env_file, &self.env_ex_file] {
            if f.exists() {
                return Err(PathAlreadyExists(f.display().to_string()));
            }
        }

        Ok(())
    }
}

/// Determines whether the path buf corresponds to an empty directory.
fn is_empty_dir(p: &PathBuf) -> Result<bool> {
    Ok(p.is_dir() && p.read_dir()?.next().is_none())
}
