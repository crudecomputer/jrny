use std::{
    fs,
    io::prelude::*,
    path::PathBuf,
    time::SystemTime,
    str::FromStr,
};

use crate::Executor;

mod begin;
use begin::Begin;

//mod review;
//pub use review::review;


/// Accepts a path string targeting a directory to set up project files:
/// The directory will be created if it does not exist or will fail if
/// pointing to an existing non-directory. This will then either verify
/// that there is an empty `revisions` directory nested within it or
/// create it if not already present. If any error occurs, any changes
/// to the file system will be attempted to be reversed.
pub fn begin(p: &str) -> Result<(), String> {
    let cmmd = Begin::new_project(p)?
        .create_root()?
        .create_revisions()?
        .create_conf()?;

    println!("The journey has begun");

    print_path("  ",     cmmd.created_root,      &cmmd.paths.root.name);
    print_path("  ├── ", cmmd.created_revisions, &cmmd.paths.revisions.name);
    print_path("  └── ", cmmd.created_conf,      &cmmd.paths.conf.name);

    Ok(())
}

/// Accepts a name for the migration file and an optional path to a config file.
/// If no path is provided, it will add a timestamped SQL file relative to current
/// working directory; otherwise it will add file in a directory relative to config.
pub fn revise(name: &str, conf_path: Option<&str>) -> Result<(), String> {
    // Non-monotonic clock should be fine since precision isn't important.
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let mut revision_path = match conf_path {
        Some(cp) => {
            let mut conf_path = PathBuf::from_str(cp).map_err(|e| e.to_string())?;

            if !conf_path.pop() {
                return Err("Config filepath is not valid".to_string());
            }

            conf_path
        },
        None => PathBuf::new(),
    };

    let filename = format!("{}-{}.sql", timestamp, name);
    revision_path.push("revisions");
    revision_path.push(&filename);

    fs::File::create(&revision_path)
        .map_err(|e| e.to_string())?
        .write_all(format!("-- Journey revision\n--\n-- {}\n--\n\n", filename).as_bytes())
        .map_err(|e| e.to_string())?;

    println!("Created {}", revision_path.display());

    Ok(())
}


//struct FileRevision {
    //applied: bool,
    //checksum: String,
    //name: String,
    //timestamp: String,
//}


pub fn review(conf_path_name: Option<&str>) -> Result<(), String> {
    let mut exec = Executor::new(conf_path_name)?;

    exec.ensure_table_exists()?;

    /*
     * Load all revisions from disk and hash each
     *
     * Load all revisions from database
     *
     * Verify all applied are still present on disk
     *
     * Verify all applied have same hash as from file
     *
     * Mark which disk revisions have not been applied
    */

    Ok(())
}


/// Prints path string with optional prefix and "[created]" suffix if the created
/// condition is true.
fn print_path(prefix: &str, created: bool, path_name: &str) {
    println!(
        "{}{}{}",
        prefix,
        path_name,
        if created { " [created]" } else { "" },
    );
}
