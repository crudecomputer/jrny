use crate::ProjectPaths;

use std::io::Write;
use std::fs;


const CONF: &str = "jrny.toml";
const CONF_TEMPLATE: &[u8] = r#"# jrny.toml

[app]
executor = "postgres"
schema = "public"
table = "jrny_revisions"

[connection]
host = "localhost"
port = 5432
name = "dbname"
user = "dbrole"
"#
.as_bytes();


pub struct Begin {
    pub paths: ProjectPaths,
    pub created_conf: bool,
    pub created_revisions: bool,
    pub created_root: bool,
}

impl Begin {
    /// Accepts a path string targeting a directory to set up project files:
    /// The directory will be created if it does not exist or will fail if
    /// pointing to an existing non-directory. This will then either verify
    /// that there is an empty `revisions` directory nested within it or
    /// create it if not already present. If any error occurs, any changes
    /// to the file system will be attempted to be reversed.
    pub fn new_project(p: &str) -> Result<Self, String> {
        Ok(Self {
            paths: ProjectPaths::for_new_project(p, CONF)?,
            created_conf: false,
            created_revisions: false,
            created_root: false,
        })
    }

    /// Attempts to create the project root directory if it doesn't exist,
    /// marking created as true if newly created.
    pub fn create_root(mut self) -> Result<Self, String> {
        if !self.paths.root.path.exists() {
            fs::create_dir(&self.paths.root.path).map_err(|e| e.to_string())?;
            self.created_root = true;
        }

        Ok(self)
    }

    /// Attempts to create the revisions directory if it doesn't exist,
    /// marking created as true if newly created.
    pub fn create_revisions(mut self) -> Result<Self, String> {
        if !self.paths.revisions.path.exists() {
            if let Err(e) = fs::create_dir(&self.paths.revisions.path) {
                self.revert()?;

                return Err(e.to_string());
            }

            self.created_revisions = true;
        }

        Ok(self)
    }

    /// Attempts to create the default configuration file. If a failure occurs,
    /// it will attempt to clean up any directory or file created during the command.
    pub fn create_conf(mut self) -> Result<Self, String> {
        let mut err = None;

        match fs::File::create(&self.paths.conf.path) {
            Ok(mut f) => {
                self.created_conf = true;

                if let Err(e) = f.write_all(CONF_TEMPLATE) {
                    err = Some(e.to_string());
                }
            }
            Err(e) => {
                err = Some(e.to_string());
            }
        }

        if let Some(e) = err {
            self.revert()?;

            return Err(e);
        }

        Ok(self)
    }

    /// Attempts to remove any directories or files created during command execution.
    fn revert(&self) -> Result<(), String> {
        if self.created_conf {
            fs::remove_file(&self.paths.conf.path).map_err(|e| e.to_string())?;
        }

        if self.created_revisions {
            fs::remove_dir(&self.paths.revisions.path).map_err(|e| e.to_string())?;
        }

        if self.created_root {
            fs::remove_dir(&self.paths.root.path).map_err(|e| e.to_string())?;
        }

        Ok(())
    }
}
