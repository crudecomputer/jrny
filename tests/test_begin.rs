use std::fs::remove_dir_all;
use jrny::begin;

mod helpers {
    use std::ffi::OsString;
    use std::fs::{DirEntry, read_to_string, remove_file};
    use std::path::{Path, PathBuf};
    use super::remove_dir_all;

    const CONF_TEMPLATE: &str = r#"# jrny config

# Project-level configuration options that should not change across environments
# or contain any sensitive information.
#
# This file MUST BE INCLUDED in version control.

# General settings for the revisions.
[revisions]

# The directory in which to store revisions, relative to this
# config file.
#
# This folder can be freely renamed or moved at any point, as long as
# the revisions within do not themselves change.
directory = "revisions"

# General settings for the database table that tracks applied revisions.
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

const ENV_TEMPLATE: &str = r#"# jrny environment

# Environment-specific configuration options, including secrets such as database
# authentication. Runtime command flags will take precedence over any values provided.
#
# This file MUST BE EXCULUDED from version control.

# General environment settings for the database connection.
[database]

# Database connection string - for permissible formats and options see:
# https://docs.rs/postgres/0.19.1/postgres/config/struct.Config.html
url = ""
"#;

const ENV_EX_TEMPLATE: &str = r#"# jrny environment EXAMPLE FILE

# This is an example file specifying optional environment-specific to include within
# a `jrny-env.toml` file. If that file is not present, `jrny` will require
# that necessary secrets are passed in via command flags.
#
# If `jrny-secret.toml` is present, runtime command flags will take precedence
# over any values contained within the file.
#
# This file SHOULD BE INCLUDED in version control.

# General environment settings for the database connection.
[database]

# Database connection string - for permissible formats and options see:
# https://docs.rs/postgres/0.19.1/postgres/config/struct.Config.html
url = "postgresql://user:password@host:port/dbname"
"#;

    pub fn test_dir(dirname: &str) -> PathBuf {
        PathBuf::from(&format!("tests/fixtures/dirs/{dirname}"))
    }

    pub fn dir_entries(dir: &Path) -> Vec<DirEntry> {
        dir
            .read_dir()
            .unwrap()
            .map(|e| e.unwrap())
            .filter(|entry| entry.file_name() != OsString::from(".gitkeep"))
            .collect()
    }

    pub fn assert_empty_directory(dir: &Path) {
        assert!(
            dir.is_dir(),
            "{} should be a directory",
            dir.display(),
        );

        let entries = dir_entries(&dir);
        assert_eq!(entries.len(), 0);
    }

    pub fn assert_file_contents_match(path: &Path, contents: &str) {
        assert_eq!(
            read_to_string(&path).unwrap(),
            contents,
        )
    }

    pub fn assert_expected_structure(dir: &Path) {
        let entries = dir_entries(&dir);

        assert_eq!(entries.len(), 4);

        for entry in entries {
            let filename = entry.file_name().into_string().unwrap();
            let path = entry.path();

            match filename.as_str() {
                "jrny.toml" => {
                    assert_file_contents_match(&path, CONF_TEMPLATE);
                },
                "jrny-env.toml" => {
                    assert_file_contents_match(&path, ENV_TEMPLATE);
                },
                "jrny-env.example.toml" => {
                    assert_file_contents_match(&path, ENV_EX_TEMPLATE);
                },
                "revisions" => {
                    assert_empty_directory(&entry.path());
                },
                _ => panic!("Unexpected file {}", filename)
            }
        }
    }

    pub fn clean_directory(dir: &Path) {
        for entry in dir_entries(dir) {
            let path = entry.path();
            let result = if path.is_file() {
                remove_file(&path)
            } else {
                remove_dir_all(&path)
            };

            result.expect(&format!("Failed to remove {}", path.display()))
        }
        assert_empty_directory(dir);
    }
}

use helpers::*;


#[test]
fn new_project_directory_works() {
    let dir = test_dir("00-nonexistent");

    // Sanity check
    assert!(!dir.exists());

    begin(&dir).unwrap();
    assert_expected_structure(&dir);

    // Clean up
    remove_dir_all(&dir).unwrap();
    assert!(!dir.exists());
}

#[test]
fn existing_empty_directory_works() {
    let dir = test_dir("01-existing-empty");

    // Make sure it's empty on each run
    assert_empty_directory(&dir);

    begin(&dir).unwrap();
    assert_expected_structure(&dir);

    clean_directory(&dir);
}

#[test]
fn existing_directory_with_empty_revisions_directory_works() {
    // dirs/02
}

#[test]
fn existing_directory_with_nonempty_revisions_directory_fails() {
    // dirs/03
}

#[test]
fn existing_nonempty_directory_works() {
    // dirs/04
}

#[test]
fn existing_nonempty_directory_with_existing_config_fails() {
    // dirs/05
}

#[test]
fn existing_nonempty_directory_with_existing_env_fails() {
    // dirs/06
}
