use chrono::{DateTime, Local, Utc};
use log::{info, warn};
use std::path::PathBuf;

pub mod commands;
pub mod paths;
pub mod revisions;

mod client;
mod config;
mod error;
mod executor;
mod logger;

pub use config::Config;
pub use error::Error;
pub use executor::Executor;
pub use logger::Logger;

pub type Result<T> = std::result::Result<T, Error>;

const CONF: &str = "jrny.toml";
const CONF_TEMPLATE: &[u8] = r#"# jrny config

[connection]
# Specifies how `jrny` will connect to the target database. Available strategies are:
#
#   env-url-string:  Use a single connection string of any supported format, the value
#                    of which is read from an environment variable whose name is specified
#                    by `var-name`
#
# NOTE: Only this single strategy is supported currently, but this is likely to be
# extended in the future.
#
strategy = { type = "env-url-string", var-name = "JRNY_DATABASE_URL" }

[table]
# Specifies which schema and table `jrny` will use to track revision history.
#
# These can freely be changed for new projects. To update these for existing projects
# with revisions already executed, you would need to first manually create the new table
# and then copy all existing revision records from the old table into the new one prior
# to running any commands with `jrny`. Otherwise, `jrny` will attempt to run all again.
schema = "public"
name = "jrny_revision"
"#
.as_bytes();

/// Accepts a path string targeting a directory to set up project files:
/// The directory will be created if it does not exist or will fail if
/// pointing to an existing non-directory. This will then either verify
/// that there is an empty `revisions` directory nested within it or
/// create it if not already present. If any error occurs, any changes
/// to the file system will be attempted to be reversed.
pub fn begin(path: &str) -> Result<()> {
    let cmd = commands::Begin::new_project(path)?;

    info!("A journey has begun");

    print_path("  ", cmd.created_root, &cmd.paths.root);
    print_path("  ├── ", cmd.created_revisions, &cmd.paths.revisions);
    print_path("  └── ", cmd.created_conf, &cmd.paths.conf);

    Ok(())
}

/// Accepts a name for the migration file and an optional path to a config file.
/// If no path is provided, it will add a timestamped SQL file relative to current
/// working directory; otherwise it will add file in a directory relative to config.
pub fn plan(name: &str, conf_path_name: Option<&str>) -> Result<()> {
    let config = Config::new(conf_path_name)?;
    let cmd = commands::Plan::new_revision(&config, name)?;

    info!("Created {}", cmd.filename);

    Ok(())
}

pub fn review(conf_path_name: Option<&str>) -> Result<()> {
    let config = Config::new(conf_path_name)?;
    let mut exec = Executor::new(&config)?;

    let cmd = commands::Review::annotated_revisions(&config, &mut exec)?;

    if cmd.revisions.is_empty() {
        info!("No revisions found. Create your first revision with `jrny plan <some-name>`.");
        return Ok(());
    }

    info!("The journey thus far\n");
    info!("{:50}{:25}{:25}", "Revision", "Created", "Applied");

    let format_local = |dt: DateTime<Utc>| DateTime::<Local>::from(dt).format("%v %X").to_string();

    for revision in cmd.revisions {
        let applied_on = match revision.applied_on {
            Some(a) => format_local(a),
            _ => "--".to_string(),
        };

        let error = if let Some(false) = revision.checksums_match {
            Some("The file has changed after being applied")
        } else if !revision.on_disk {
            Some("No corresponding file could not be found")
        } else {
            None
        };

        match error {
            Some(error) => warn!(
                "{:50}{:25}{:25}{}",
                revision.filename,
                format_local(revision.created_at),
                applied_on,
                error,
            ),
            None => info!(
                "{:50}{:25}{:25}",
                revision.filename,
                format_local(revision.created_at),
                applied_on,
            ),
        }
    }

    Ok(())
}

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
fn print_path(prefix: &str, created: bool, path: &PathBuf) {
    info!(
        "{}{}{}",
        prefix,
        path.display(),
        if created { " [created]" } else { "" },
    );
}
