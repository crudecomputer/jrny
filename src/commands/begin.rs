use std::{fs, io::Write};

use crate::{
    project::ProjectPaths,
    CONF_TEMPLATE,
    ENV,
    ENV_TEMPLATE,
    ENV_EX,
    ENV_EX_TEMPLATE,
    Result,
};

#[derive(Debug)]
pub(super) struct Begin {
    // TODO This has gotten gross
    pub(super) paths: ProjectPaths,
    pub(super) created_revisions: bool,
    pub(super) created_root: bool,
    pub(super) created_conf: bool,
    pub(super) created_env: bool,
    pub(super) created_env_ex: bool,
}

impl Begin {
    pub(super) fn new(paths: ProjectPaths) -> Self {
        Self {
            paths,
            created_conf: false,
            created_env: false,
            created_env_ex: false,
            created_revisions: false,
            created_root: false,
        }
    }

    pub(super) fn create_root(&mut self) -> Result<()> {
        // Attempts to create the project root directory if it doesn't exist.
        if !self.paths.root_dir.exists() {
            fs::create_dir(&self.paths.root_dir)?;
            self.created_root = true;
        }

        Ok(())
    }

    pub(super) fn create_revisions(&mut self) -> Result<()> {
        // Attempts to create the revisions directory if it doesn't exist.
        if !self.paths.revisions_dir.exists() {
            fs::create_dir(&self.paths.revisions_dir)?;
            self.created_revisions = true;
        }

        Ok(())
    }

    pub(super) fn create_conf(&mut self) -> Result<()> {
        // Attempts to create the default configuration file.
        //
        // This will truncate the file if it exists, so it relies on
        // the paths having already checked that the file isn't present.
        //
        // TODO Clean up that logic.
        let mut f = fs::File::create(&self.paths.conf_file)?;
        self.created_conf = true;
        f.write_all(CONF_TEMPLATE.as_bytes())?;

        Ok(())
    }

    pub(super) fn create_env(&mut self) -> Result<()> {
        // Attempts to create the default environment file & example file.
        let mut f = fs::File::create(&self.paths.env_file)?;
        self.created_env = true;
        f.write_all(ENV_TEMPLATE.as_bytes())?;

        let mut f = fs::File::create(&self.paths.env_ex_file)?;
        self.created_env_ex = true;
        f.write_all(ENV_EX_TEMPLATE.as_bytes())?;

        Ok(())
    }

    pub(super) fn revert(&self) -> Result<()> {
        // Attempts to remove any directories or files created during command execution.
        if self.created_conf {
            fs::remove_file(&self.paths.conf_file)?;
        }

        if self.created_env {
            fs::remove_file(&self.paths.env_file)?;
        }

        if self.created_env_ex {
            fs::remove_file(&self.paths.env_ex_file)?;
        }

        if self.created_revisions {
            fs::remove_dir(&self.paths.revisions_dir)?;
        }

        if self.created_root {
            fs::remove_dir(&self.paths.root_dir)?;
        }

        Ok(())
    }
}
