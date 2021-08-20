use std::{fs, io::Write};

use crate::{
    paths::ProjectPaths,
    Result,
    CONF_TEMPLATE,
    ENV_TEMPLATE,
    ENV_EX,
    ENV_EX_TEMPLATE,
};

pub struct Begin {
    pub paths: ProjectPaths,

    // TODO This has gotten gross
    pub created_revisions: bool,
    pub created_root: bool,
    created_conf: bool,
    created_env: bool,
    created_env_example: bool,
}

impl Begin {
    /// Accepts a path string targeting a directory to set up project files:
    /// The directory will be created if it does not exist or will fail if
    /// pointing to an existing non-directory. This will then either verify
    /// that there is an empty `revisions` directory nested within it or
    /// create it if not already present. If any error occurs, any changes
    /// to the file system will be attempted to be reversed.
    pub fn new_project(p: &str) -> Result<Self> {
        let mut cmd = Self {
            paths: ProjectPaths::for_new_project(p)?,
            created_conf: false,
            created_env: false,
            created_env_example: false,
            created_revisions: false,
            created_root: false,
        };

        cmd.create_root()
            .and_then(|_| cmd.create_revisions())
            .and_then(|_| cmd.create_conf())
            .and_then(|_| cmd.create_env())
            .map_err(|e| match cmd.revert() {
                Ok(_) => e,
                // TODO there should really be better error handling, since this
                // will lose the original error
                Err(err) => err,
            })?;

        Ok(cmd)
    }

    /// Attempts to create the project root directory if it doesn't exist,
    /// marking created as true if newly created.
    fn create_root(&mut self) -> Result<()> {
        if !self.paths.root.exists() {
            fs::create_dir(&self.paths.root)?;
            self.created_root = true;
        }

        Ok(())
    }

    /// Attempts to create the revisions directory if it doesn't exist.
    fn create_revisions(&mut self) -> Result<()> {
        if !self.paths.revisions.exists() {
            fs::create_dir(&self.paths.revisions)?;
            self.created_revisions = true;
        }

        Ok(())
    }

    /// Attempts to create the default configuration file.
    fn create_conf(&mut self) -> Result<()> {
        // This will truncate the file if it exists, so it relies on
        // the paths having already checked that the file isn't present.
        //
        // TODO Clean up that logic.
        let mut f = fs::File::create(&self.paths.conf)?;
        self.created_conf = true;
        f.write_all(CONF_TEMPLATE.as_bytes())?;

        Ok(())
    }

    /// Attempts to create the default environment file.
    fn create_env(&mut self) -> Result<()> {
        let mut f = fs::File::create(&self.paths.env)?;
        self.created_env = true;
        f.write_all(ENV_TEMPLATE.as_bytes())?;

        // This isn't stored on `paths` because the file is completely ignored
        // on every command other than `begin`
        let mut f = fs::File::create(&self.paths.root.join(ENV_EX))?;
        self.created_env_example = true;
        f.write_all(ENV_EX_TEMPLATE.as_bytes())?;

        Ok(())
    }

    /// Attempts to remove any directories or files created during command execution.
    fn revert(&self) -> Result<()> {
        if self.created_conf {
            fs::remove_file(&self.paths.conf)?;
        }

        if self.created_env {
            fs::remove_file(&self.paths.env)?;
        }
        if self.created_env_example {
            fs::remove_file(&self.paths.root.join(ENV_EX))?;
        }

        if self.created_revisions {
            fs::remove_dir(&self.paths.revisions)?;
        }

        if self.created_root {
            fs::remove_dir(&self.paths.root)?;
        }

        Ok(())
    }
}
