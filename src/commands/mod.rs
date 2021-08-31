use std::{
    fs,
    io::Write,
    path::PathBuf,
};

use chrono::Utc;
use log::info;

use crate::{
    project::ProjectPaths,
    revisions::RevisionFile,
    Result,
};

mod begin;

use begin::Begin;

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
