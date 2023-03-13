use std::fs;
use std::io::Write;
use std::path::Path;

use chrono::{DateTime, Local, Utc};
use log::{info, warn};

use crate::context::{Config, Environment};
use crate::revisions::RevisionFile;
use crate::{Error, Executor, Result};

mod begin;
// mod embark;
mod review;

use begin::Begin;
// use embark::Embark;
use review::check_revisions;

pub use review::RevisionSummary;

/// Accepts a path string targeting a directory to set up project files:
/// The directory will be created if it does not exist or will fail if
/// pointing to an existing non-directory. This will then either verify
/// that there is an empty `revisions` directory nested within it or
/// create it if not already present. If any error occurs, any changes
/// to the file system will be attempted to be reversed.
pub fn begin(dirpath: &Path) -> Result<()> {
    let mut cmd = Begin::new(dirpath)?;

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

    info!("A journey has begun");

    // Only some paths could have been created by the command, so those
    // need to be dynamically labeled as such.
    log_path("  ", &cmd.paths.root_dir, cmd.created_root);
    log_path("  ├── ", &cmd.paths.revisions_dir, cmd.created_revisions);
    log_path("  └── ", &cmd.paths.conf_file, true);
    log_path("  └── ", &cmd.paths.env_file, true);
    log_path("  └── ", &cmd.paths.env_ex_file, true);

    Ok(())
}

/// Generates a new empty revision file with the given name in the
/// revisions directory specified by the provided config.
pub fn plan(cfg: &Config, name: &str, contents: Option<&str>) -> Result<()> {
    let timestamp = Utc::now().timestamp();
    let next_id = RevisionFile::all(&cfg.revisions.directory)?
        .iter()
        .reduce(|rf1, rf2| if rf1.id > rf2.id { rf1 } else { rf2 })
        .map_or(0, |rf| rf.id)
        + 1;

    let new_filename = format!("{:03}.{}.{}.sql", next_id, timestamp, name);
    let new_path = cfg.revisions.directory.join(new_filename);

    let contents = contents.map(|c| c.to_owned()).unwrap_or_else(|| {
        format!(
            "-- Revision: {name}
--
-- Add description here

begin;

-- Add SQL here

commit;
",
            name = name
        )
    });

    fs::File::create(&new_path)?.write_all(contents.as_bytes())?;

    info!("Created {}", new_path.display());

    Ok(())
}

/// Reviews the status of all revisions specified by the config as well as
/// their status in the database.
pub fn review(cfg: &Config, env: &Environment) -> Result<()> {
    let mut exec = Executor::new(cfg, env)?;
    let review = check_revisions(&mut exec, &cfg.revisions.directory)?;

    if review.revisions.is_empty() {
        info!("No revisions found. Create your first revision with `jrny plan <some-name>`.");
        return Ok(());
    }

    info!("The journey thus far:");

    for rev in &review.revisions {
        info!("");
        info!("  [{}] {}", rev.meta.id, rev.meta.name);
        info!("    Created on {}", format_local(rev.meta.created_at));

        if let Some(applied_on) = rev.meta.applied_on {
            info!("    Applied on {}", format_local(applied_on));
        }

        if !rev.problems.is_empty() {
            warn!("    Errors:");
            for prob in &rev.problems {
                warn!("      - {}", prob);
            }
        }
    }

    if review.failed() {
        return Err(Error::RevisionsFailedReview(review.summary));
    }

    Ok(())
}

/// Applies all pending revisions specified by the given config to the
/// database specified by the environment.
pub fn embark(cfg: &Config, env: &Environment) -> Result<()> {
    let mut exec = Executor::new(cfg, env)?;

    let cmd = Embark::prepare(cfg, &mut exec)?;

    if cmd.to_apply.is_empty() {
        info!("No revisions to apply");
        return Ok(());
    }

    cmd.apply(&mut exec)?;

    Ok(())
}

/// Logs the path string with optional prefix and "[created]" suffix if the created
/// condition is true.
fn log_path(prefix: &str, path: &Path, created: bool) {
    info!(
        "{}{}{}",
        prefix,
        path.display(),
        if created { " [created]" } else { "" },
    );
}

fn format_local(dt: DateTime<Utc>) -> String {
    DateTime::<Local>::from(dt).format("%v %X").to_string()
}
