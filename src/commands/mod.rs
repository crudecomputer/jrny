use std::{fs, io::Write, path::PathBuf};

use chrono::{DateTime, Local, Utc};
use log::{info, warn};

use crate::{executor::Executor, revisions::RevisionFile, Config, Environment, Result};

mod begin;
mod embark;
mod review;

use begin::Begin;
use embark::Embark;
use review::Review;

/// Accepts a path string targeting a directory to set up project files:
/// The directory will be created if it does not exist or will fail if
/// pointing to an existing non-directory. This will then either verify
/// that there is an empty `revisions` directory nested within it or
/// create it if not already present. If any error occurs, any changes
/// to the file system will be attempted to be reversed.
pub fn begin(dirpath: &PathBuf) -> Result<()> {
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
pub fn plan(cfg: &Config, name: &str) -> Result<()> {
    let timestamp = Utc::now().timestamp();
    let next_id = RevisionFile::all_from_disk(&cfg.revisions.directory)?
        .iter()
        .reduce(|rf1, rf2| if rf1.id > rf2.id { rf1 } else { rf2 })
        .map_or(0, |rf| rf.id as i32)
        + 1;

    let new_filename = format!("{:03}.{}.{}.sql", next_id, timestamp, name);
    let new_path = cfg.revisions.directory.join(&new_filename);

    let contents = format!(
        "-- Revision: {name}
--
-- Add description here

begin;

-- Add SQL here

commit;
",
        name = name
    );

    fs::File::create(&new_path)?.write_all(contents.as_bytes())?;

    info!("Created {}", new_path.display());

    Ok(())
}

/// Reviews the status of all revisions specified by the config as well as
/// their status in the database.
pub fn review(cfg: &Config, env: &Environment) -> Result<()> {
    let mut exec = Executor::new(&cfg, &env)?;
    let cmd = Review::annotated_revisions(&mut exec, &cfg.revisions.directory)?;

    if cmd.revisions.is_empty() {
        info!("No revisions found. Create your first revision with `jrny plan <some-name>`.");
        return Ok(());
    }

    info!("The journey thus far\n");
    info!(
        "  {:3}  {:43}{:25}{:25}",
        "Id", "Revision", "Created", "Applied"
    );

    let format_local = |dt: DateTime<Utc>| DateTime::<Local>::from(dt).format("%v %X").to_string();

    let mut last_applied_index = -1;

    for (i, revision) in cmd.revisions.iter().enumerate() {
        if revision.applied_on.is_some() {
            last_applied_index = i as isize;
        }
    }

    // TODO clean up? this isn't elegant
    let mut previous_id = None;

    for (i, revision) in cmd.revisions.iter().enumerate() {
        let applied_on = match revision.applied_on {
            Some(a) => format_local(a),
            _ => "--".to_string(),
        };

        let error = if let Some(false) = revision.checksums_match {
            Some("The file has changed after being applied")
        } else if !revision.on_disk {
            Some("No corresponding file could not be found")
        } else if revision.applied_on.is_none() && (i as isize) < last_applied_index {
            Some("Later revisions have already been applied")
        } else if previous_id == Some(revision.id) {
            Some("Revision has duplicate id")
        } else {
            None
        };

        match error {
            Some(error) => warn!(
                "  {:3}  {:43}{:25}{:25}{}",
                revision.id,
                revision.name,
                format_local(revision.created_at),
                applied_on,
                error,
            ),
            None => info!(
                "  {:3}  {:43}{:25}{:25}",
                revision.id,
                revision.name,
                format_local(revision.created_at),
                applied_on,
            ),
        }

        previous_id = Some(revision.id);
    }

    Ok(())
}

/// Applies all pending revisions specified by the given config to the
/// database specified by the environment.
pub fn embark(cfg: &Config, env: &Environment) -> Result<()> {
    let mut exec = Executor::new(&cfg, &env)?;

    let cmd = Embark::prepare(&cfg, &mut exec)?;

    if cmd.to_apply.is_empty() {
        info!("No revisions to apply");
        return Ok(());
    }

    cmd.apply(&mut exec)?;

    Ok(())
}

/// Logs the path string with optional prefix and "[created]" suffix if the created
/// condition is true.
fn log_path(prefix: &str, path: &PathBuf, created: bool) {
    info!(
        "{}{}{}",
        prefix,
        path.display(),
        if created { " [created]" } else { "" },
    );
}
