use chrono::{DateTime, Local, Utc};
use log::{info, warn};

pub mod commands;

mod client;
mod error;
mod executor;
mod logger;
mod project;
mod revisions;

pub use error::Error;
pub use logger::Logger;


pub type Result<T> = std::result::Result<T, error::Error>;

pub const CONF: &str = "jrny.toml";
const CONF_TEMPLATE: &str = r#"# jrny config

# Project-level configuration options that should not change across environments
# or contain any sensitive information.
#
# This file MUST BE INCLUDED in version control.

[project]
# The directory in which to store revisions.
#
# This folder can be freely renamed or moved at any point, as long as
# the revisions within do not themselves change.
revisions_directory = "revisions"

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

pub const ENV: &str = "jrny-env.toml";
const ENV_TEMPLATE: &str = r#"# jrny environment

# Environment-specific configuration options, including secrets such as database
# authentication. Runtime command flags will take precedence over any values provided.
#
# This file MUST BE EXCULUDED from version control.

[database]
url = ""
"#;

pub const ENV_EX: &str = "jrny-env.example.toml";
const ENV_EX_TEMPLATE: &str = r#"# jrny environment EXAMPLE FILE

# This is an example file specifying optional environment-specific to include within
# a `jrny-env.toml` file. If that file is not present, `jrny` will require
# that necessary secrets are passed in via command flags.
#
# If `jrny-secret.toml` is present, runtime command flags will take precedence
# over any values contained within the file.
#
# This file SHOULD BE INCLUDED in version control.

[database]

# Database connection string - for permissible formats and options see:
# https://docs.rs/postgres/0.19.1/postgres/config/struct.Config.html
url = "postgresql://user:password@host:port/dbname"
"#;

/*

pub fn embark(conf_path_name: Option<&str>) -> Result<()> {
    let config = Config::new(conf_path_name)?;
    let mut exec = Executor::new(&config)?;

    let cmd = commands::Embark::prepare(&config, &mut exec)?;

    if cmd.to_apply.is_empty() {
        info!("No revisions to apply");
        return Ok(());
    }

    cmd.apply(&mut exec)?;

    Ok(())
}

/// Prints path string with optional prefix and "[created]" suffix if the created
/// condition is true.
fn print_path(prefix: &str, path: &PathBuf, created: bool) {
    info!(
        "{}{}{}",
        prefix,
        path.display(),
        if created { " [created]" } else { "" },
    );
}
*/
