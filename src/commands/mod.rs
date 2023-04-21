use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Local, Utc};
use log::{info, warn};

use crate::context::{Config, Environment};
use crate::revisions::RevisionFile;
use crate::{Error, Executor, Result};

mod begin;
mod review;

use begin::Begin;
use review::Review;

pub use review::ReviewSummary;

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
pub fn plan(cfg: &Config, name: &str, contents: Option<&str>) -> Result<PathBuf> {
    // TODO: Is second-precision good enough?
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
    Ok(new_path)
}

/// Reviews the status of all revisions specified by the config as well as
/// their status in the database.
pub fn review(cfg: &Config, env: &Environment) -> Result<()> {
    let mut exec = Executor::new(&cfg.table, env)?;
    let review = Review::new(&mut exec, &cfg.revisions.directory)?;

    if review.items().is_empty() {
        info!("No revisions found. Create your first revision with `jrny plan <some-name>`.");
        return Ok(());
    }

    info!("The journey thus far:");

    for item in review.items() {
        info!("");
        info!("  [{}] {}", item.id(), item.name());
        info!("    Created on {}", format_local(*item.created_at()));

        if let Some(applied_on) = item.applied_on() {
            info!("    Applied on {}", format_local(*applied_on));
        }

        if !item.problems().is_empty() {
            warn!("    Errors:");
            for prob in item.problems() {
                warn!("      - {}", prob);
            }
        }
    }

    if review.failed() {
        return Err(Error::RevisionsFailedReview(review.summary().to_owned()));
    }

    Ok(())
}

/// Applies all pending revisions specified by the given config to the
/// database specified by the environment.
pub fn embark(cfg: &Config, env: &Environment, through_id: Option<i32>) -> Result<()> {
    let mut exec = Executor::new(&cfg.table, env)?;
    let review = Review::new(&mut exec, &cfg.revisions.directory)?;

    if review.failed() {
        return Err(Error::RevisionsFailedReview(review.summary().to_owned()));
    }

    let pending = review.pending_revisions();

    if pending.is_empty() {
        info!("No revisions to apply");
        return Ok(());
    }

    let to_apply = match through_id {
        Some(through_id) => {
            let to_apply: Vec<&RevisionFile> = pending
                .iter()
                .filter(|rev| rev.id <= through_id)
                .copied()
                .collect();
            let to_skip: Vec<&RevisionFile> = pending
                .iter()
                .filter(|rev| rev.id > through_id)
                .copied()
                .collect();

            match (to_apply.as_slice(), to_skip.as_slice()) {
                ([], []) => unreachable!("pending revisions should not be empty"),
                (_, []) => {
                    info!("Applying {} revision(s)", to_apply.len());
                }
                ([], _) => {
                    info!(
                        "No revisions to apply, skipping {} revision(s)",
                        to_skip.len()
                    );
                }
                _ => {
                    info!(
                        "Applying {} revision(s), skipping {}",
                        to_apply.len(),
                        to_skip.len()
                    );
                }
            }

            to_apply
        }
        None => {
            info!("Applying {} revision(s)", pending.len());
            pending
        }
    };

    if !to_apply.is_empty() {
        info!("");
        for revision in &to_apply {
            info!("  {}", revision.filename);
            exec.run_revision(revision)?;
        }
    }

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
