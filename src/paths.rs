//! Utilities for working with paths.
use std::path::PathBuf;

use crate::{CONF, Error, Result};

/// A container for the various paths of interest for a project.
pub struct ProjectPaths {
    pub conf: PathBuf,
    pub revisions: PathBuf,
    pub root: PathBuf,
}

impl ProjectPaths {
    /// Creates path bufs for a new project given a root directory, ensuring that
    /// there is not already a config file or a non-empty revisions directory.
    pub fn for_new_project(root_dir: &str) -> Result<Self> {
        let root = PathBuf::from(root_dir);
        let revisions = root.join("revisions");
        let conf = root.join(CONF);

        let paths = Self {
            conf,
            revisions,
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
            .ok_or_else(|| Error::InvalidPath(conf.display().to_string()))?
            .to_path_buf();

        let revisions = root.join("revisions");

        Ok(Self {
            conf,
            revisions,
            root,
        })
    }

    /// Ensures that own path bufs are valid for a new project, namely that the
    /// root path is a directory if exists, that the revisions directory is empty
    /// if exists, and that no config file exists.
    fn valid_for_new(&self) -> Result<()> {
        use Error::*;

        if self.root.exists() && !self.root.is_dir() {
            return Err(NotDirectory(self.root.display().to_string()));
        }

        if self.revisions.exists() && !Self::is_empty_dir(&self.revisions)? {
            return Err(NotEmptyDirectory(self.revisions.display().to_string()));
        }

        if self.conf.exists() {
            return Err(AlreadyExists(self.conf.display().to_string()));
        }

        Ok(())
    }

    /// Determines whether the path buf corresponds to an empty directory.
    fn is_empty_dir(p: &PathBuf) -> Result<bool> {
        Ok(
            p.is_dir() &&
            p.read_dir()?.next().is_none()
        )
    }
}
