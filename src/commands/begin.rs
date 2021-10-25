use std::{
    fs,
    io::Write,
    path::PathBuf,
};

use crate::{
    CONF,
    ENV,
    ENV_EX,
    Error,
    Result,
};

const CONF_TEMPLATE: &str = r#"# jrny config

# Project-level configuration options that should not change across environments
# or contain any sensitive information.
#
# This file MUST BE INCLUDED in version control.

# General settings for the revisions.
[revisions]

# The directory in which to store revisions, relative to this
# config file.
#
# This folder can be freely renamed or moved at any point, as long as
# the revisions within do not themselves change.
directory = "revisions"

# General settings for the database table that tracks applied revisions.
[table]

# Specifies which schema and table `jrny` will use to track revision history.
#
# These can freely be changed for new projects. To update these for existing projects
# with revisions already executed, you would need to first manually create the new table
# and then copy all existing revision records from the old table into the new one prior
# to running any commands with `jrny`. Otherwise, `jrny` will attempt to run all again.
schema = "public"
name = "jrny_revision"
"#;

const ENV_TEMPLATE: &str = r#"# jrny environment

# Environment-specific configuration options, including secrets such as database
# authentication. Runtime command flags will take precedence over any values provided.
#
# This file MUST BE EXCULUDED from version control.

# General environment settings for the database connection.
[database]

# Database connection string - for permissible formats and options see:
# https://docs.rs/postgres/0.19.1/postgres/config/struct.Config.html
url = ""
"#;

const ENV_EX_TEMPLATE: &str = r#"# jrny environment EXAMPLE FILE

# This is an example file specifying optional environment-specific to include within
# a `jrny-env.toml` file. If that file is not present, `jrny` will require
# that necessary secrets are passed in via command flags.
#
# If `jrny-secret.toml` is present, runtime command flags will take precedence
# over any values contained within the file.
#
# This file SHOULD BE INCLUDED in version control.

# General environment settings for the database connection.
[database]

# Database connection string - for permissible formats and options see:
# https://docs.rs/postgres/0.19.1/postgres/config/struct.Config.html
url = "postgresql://user:password@host:port/dbname"
"#;

fn is_empty_dir(p: &PathBuf) -> Result<bool> {
    Ok(p.is_dir() && p.read_dir()?.next().is_none())
}


#[derive(Debug)]
pub(super) struct BeginPaths {
    pub conf_file: PathBuf,
    pub env_file: PathBuf,
    pub env_ex_file: PathBuf,
    pub revisions_dir: PathBuf,
    pub root_dir: PathBuf,
}

impl BeginPaths {
    pub fn new(root_dir: &PathBuf) -> Result<Self> {
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

#[derive(Debug)]
pub(super) struct Begin {
    pub(super) created_revisions: bool,
    pub(super) created_root: bool,
    pub(super) paths: BeginPaths,
    created_conf: bool,
    created_env: bool,
    created_env_ex: bool,
}

impl Begin {
    pub(super) fn new(project_directory: &PathBuf) -> Result<Self> {
        let paths = BeginPaths::new(project_directory)?;

        Ok(Self {
            paths,
            created_conf: false,
            created_env: false,
            created_env_ex: false,
            created_revisions: false,
            created_root: false,
        })
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
