//! Utilities for working with paths.
use std::path::PathBuf;

use crate::{Error, Result, CONF, ENV};

/// A container for the various paths of interest for a project.
#[derive(Debug)]
pub struct ProjectPaths {
    pub conf: PathBuf,
    pub env: PathBuf,
    pub revisions: PathBuf,
    pub root: PathBuf,
}

impl ProjectPaths {
    /// Creates path bufs for a new project given a root directory, ensuring that
    /// there is not already a config & env file or a non-empty revisions directory.
    pub fn for_new_project(root_dir: &str) -> Result<Self> {
        let root = PathBuf::from(root_dir);

        let paths = Self {
            conf: root.join(CONF),
            env: root.join(ENV),
            revisions: root.join("revisions"),
            root,
        };

        paths.valid_for_new()?;

        Ok(paths)
    }

    /// Creates path bufs for a project either relative to the given config filepath name
    /// or to the current working directory if no path name is provided.
    pub fn from_conf_path(conf_path_name: Option<&str>) -> Result<Self> {
        let conf = PathBuf::from(conf_path_name.unwrap_or(CONF));

        let root = conf
            .parent()
            .ok_or_else(|| Error::PathInvalid(conf.display().to_string()))?
            .to_path_buf();

        Ok(Self {
            conf,
            env: root.join(ENV),
            revisions: root.join("revisions"),
            root,
        })
    }

    /// Ensures that own path bufs are valid for a new project, namely that the
    /// root path is a directory if exists, that the revisions directory is empty
    /// if exists, and that no config or environment file exists
    fn valid_for_new(&self) -> Result<()> {
        use Error::*;

        if self.root.exists() && !self.root.is_dir() {
            return Err(PathNotDirectory(self.root.display().to_string()));
        }

        if self.revisions.exists() && !Self::is_empty_dir(&self.revisions)? {
            return Err(PathNotEmptyDirectory(self.revisions.display().to_string()));
        }

        if self.conf.exists() {
            return Err(PathAlreadyExists(self.conf.display().to_string()));
        }

        if self.env.exists() {
            return Err(PathAlreadyExists(self.env.display().to_string()));
        }

        Ok(())
    }

    /// Determines whether the path buf corresponds to an empty directory.
    fn is_empty_dir(p: &PathBuf) -> Result<bool> {
        Ok(p.is_dir() && p.read_dir()?.next().is_none())
    }
}
