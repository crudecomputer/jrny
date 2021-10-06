use std::{
    fs,
    io::Write,
    path::PathBuf,
};

use chrono::{DateTime, Local, Utc};
use log::{info, warn};

use crate::{
    executor::Executor,
    project::{ProjectConfig, ProjectEnvironment, ProjectPaths},
    revisions::RevisionFile,
    Result,
};

mod begin;
mod review;

use begin::Begin;
use review::Review;

/// Accepts a path string targeting a directory to set up project files:
/// The directory will be created if it does not exist or will fail if
/// pointing to an existing non-directory. This will then either verify
/// that there is an empty `revisions` directory nested within it or
/// create it if not already present. If any error occurs, any changes
/// to the file system will be attempted to be reversed.
pub fn begin(dirpath: &PathBuf) -> Result<()> {
    let paths = ProjectPaths::for_new_project(&dirpath)?;

    let mut cmd = Begin::new(paths);

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

    print_path("  ",     &cmd.paths.root_dir,      cmd.created_root);
    print_path("  ├── ", &cmd.paths.revisions_dir, cmd.created_revisions);
    print_path("  └── ", &cmd.paths.conf_file,     cmd.created_conf);
    print_path("  └── ", &cmd.paths.env_file,      cmd.created_env);
    print_path("  └── ", &cmd.paths.env_ex_file,   cmd.created_env_ex);

    Ok(())
}

/// Accepts a name for the migration file and an optional path to a config file.
/// If no path is provided, it will add a timestamped SQL file relative to current
/// working directory; otherwise it will add file in a directory relative to config.
pub fn plan(confpath: &PathBuf, name: &str) -> Result<()> {
    let paths = ProjectPaths::from_conf(confpath)?;
    let timestamp = Utc::now().timestamp();
    let next_id = RevisionFile::all_from_disk(&paths.revisions_dir)?
        .iter()
        .reduce(|rf1, rf2| if rf1.id > rf2.id { rf1 } else { rf2 })
        .map_or(0, |rf| rf.id as i32)
        + 1;

    let new_filename = format!("{:03}.{}.{}.sql", next_id, timestamp, name);
    let new_path = paths.revisions_dir.join(&new_filename);

    let template = "-- :filename

begin;
-- Start revisions


-- End revisions
commit;
";

    fs::File::create(&new_path)?
        .write_all(template.replace(":filename", &new_filename).as_bytes())?;

    info!("Created {}", new_path.display());
    
    Ok(())
}

pub struct ReviewArgs {
    /// Path to TOML configuration file
    pub confpath: PathBuf,

    /// Database connection string
    pub database_url: Option<String>,

    /// Path to optional TOML environment file
    pub envpath: PathBuf,
}

pub fn review(args: ReviewArgs) -> Result<()> {
    let ReviewArgs { confpath, database_url, envpath } = args;

    let config = ProjectConfig::new(&confpath)?;
    let env = ProjectEnvironment::new(&envpath, database_url)?;
    let paths = ProjectPaths::from_conf(&confpath)?;

    let mut exec = Executor::new(&config, &env)?;
    let cmd = Review::annotated_revisions(&paths, &mut exec)?;

    if cmd.revisions.is_empty() {
        info!("No revisions found. Create your first revision with `jrny plan <some-name>`.");
        return Ok(());
    }

    info!("The journey thus far\n");
    info!(
        "  {:3}  {:43}{:25}{:25}",
        "Id", "Revision", "Created", "Applied"
    );

    let format_local = |dt: DateTime<Utc>| DateTime::<Local>::from(dt)
        .format("%v %X")
        .to_string();

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

fn print_path(prefix: &str, path: &PathBuf, created: bool) {
    // Prints path string with optional prefix and "[created]" suffix if the created
    // condition is true.
    info!(
        "{}{}{}",
        prefix,
        path.display(),
        if created { " [created]" } else { "" },
    );
}
