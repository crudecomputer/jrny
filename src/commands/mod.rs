use chrono::{DateTime, Utc};
use std::{
    collections::HashMap,
    fmt::Display,
    fs,
    io::prelude::*,
};

use crate::{Config, Executor, DatabaseRevision, FileRevision};

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

#[derive(Debug, Eq, Hash, PartialEq)]
struct RevisionIdentifier(String);

impl From<Rc<FileRevision>> for RevisionIdentifier {
    fn from(fr: Rc<FileRevision>) -> Self {
        Self(fr.filename.clone())
    }
}

impl From<Rc<DatabaseRevision>> for RevisionIdentifier {
    fn from(dr: Rc<DatabaseRevision>) -> Self {
        Self(dr.filename.clone())
    }
}


use std::rc::Rc;

#[derive(Debug)]
struct Reviewer {
    files: Vec<Rc<FileRevision>>,
    records: Vec<Rc<DatabaseRevision>>,
    files_map: HashMap<RevisionIdentifier, Rc<FileRevision>>,
    records_map: HashMap<RevisionIdentifier, Rc<DatabaseRevision>>,
}

impl Reviewer {
    fn new(mut files: Vec<FileRevision>, mut records: Vec<DatabaseRevision>) -> Self {
        let files: Vec<Rc<FileRevision>> = files
            .drain(..)
            .map(|fr| Rc::new(fr))
            .collect();

        let files_map = files.iter()
            .map(|fr| (RevisionIdentifier(fr.filename.clone()), fr.clone()))
            .collect();

        let records: Vec<Rc<DatabaseRevision>> = records
            .drain(..)
            .map(|dr| Rc::new(dr))
            .collect();

        let records_map = records.iter()
            .map(|dr| (RevisionIdentifier(dr.filename.clone()), dr.clone()))
            .collect();

        Self { files, files_map, records, records_map }
    }

    /// Checks whether the revision has been applied and, if so, whether the
    /// checksums match.
    fn check_revision_files(self) -> Self {
        self
    }

    /// Checks that each revision database record still has a corresponding
    /// revision file.
    fn check_revision_records(self) -> Self {
        self
    }
}


pub fn review(conf_path_name: Option<&str>) -> Result<(), String> {
    let config = Config::new(conf_path_name)?;
    let mut exec = Executor::new(&config)?;

    exec.ensure_table_exists()?;

    // Load all revisions from disk and hash each
    let file_revisions = FileRevision::all_from_disk(&config.paths.revisions)?;

    // Load all revisions from database
    let db_revisions = exec.load_revisions()?;

    let reviewer = Reviewer::new(file_revisions, db_revisions)
        .check_revision_files()
        .check_revision_records();

    println!("{:#?}", reviewer);

    //compare

    /*
     *
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
fn print_path(prefix: &str, created: bool, path_name: impl Display) {
    println!(
        "{}{}{}",
        prefix,
        path_name,
        if created { " [created]" } else { "" },
    );
}
