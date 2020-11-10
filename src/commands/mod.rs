use chrono::{DateTime, Utc};
use std::{
    fmt::Display,
    fs,
    io::prelude::*,
};

use crate::{Config, Executor, FileRevision};

mod begin;
use begin::Begin;

mod review;
use review::Review;


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

    print_path("  ",     cmmd.created_root,      &cmmd.paths.root.display());
    print_path("  ├── ", cmmd.created_revisions, &cmmd.paths.revisions.display());
    print_path("  └── ", cmmd.created_conf,      &cmmd.paths.conf.display());

    Ok(())
}

/// Accepts a name for the migration file and an optional path to a config file.
/// If no path is provided, it will add a timestamped SQL file relative to current
/// working directory; otherwise it will add file in a directory relative to config.
pub fn revise(name: &str, conf_path_name: Option<&str>) -> Result<(), String> {
    let config = Config::new(conf_path_name)?;

    let utc: DateTime<Utc> = Utc::now();
    let timestamp = utc.timestamp();

    let revision_path = config.paths.revisions
        .join(format!("{}.{}.sql", timestamp, name));
    let filename = revision_path.display();

    fs::File::create(&revision_path)
        .map_err(|e| e.to_string())?
        .write_all(format!(
            "-- Journey revision\n--\n-- {}\n--\n\n",
            filename,
        ).as_bytes())
        .map_err(|e| e.to_string())?;

    println!("Created {}", filename);

    Ok(())
}

pub fn review(conf_path_name: Option<&str>) -> Result<(), String> {
    let config = Config::new(conf_path_name)?;
    let mut exec = Executor::new(&config)?;

    exec.ensure_table_exists()?;

    let files = FileRevision::all_from_disk(&config.paths.revisions)?;
    let records = exec.load_revisions()?;
    let annotated = Review::annotate(files, records);

    //println!("{:#?}", annotated);

    println!("The journey thus far");
    println!("{:50}{:50}{}", "Revision name", "Applied on", "Error");

    for anno in annotated {
        println!(
            "{:50}{:50}{}",
            anno.filename,
            if let Some(a) = anno.applied_on {
                a.to_string()
            } else {
                "Not applied".to_string()
            },
            if let Some(false) = anno.checksums_match {
                "The file has changed after being applied"
            } else if !anno.on_disk {
                "No corresponding file could not be found"
            } else {
                ""
            },
        );
    }

    Ok(())
}

/// Prints path string with optional prefix and "[created]" suffix if the created
/// condition is true.
fn print_path(prefix: &str, created: bool, path_name: impl Display) {
    println!(
        "{}{}{}",
        prefix,
        path_name,
        if created { " [created]" } else { "" },
    );
}
