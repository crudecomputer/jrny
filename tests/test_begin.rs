use jrny::{begin, Error};
use std::ffi::OsString;
use std::fs::remove_dir_all;

mod helpers {
    use super::remove_dir_all;
    use jrny::{CONF, ENV, ENV_EX};
    use std::ffi::OsString;
    use std::fs::{read_to_string, remove_file, DirEntry};
    use std::path::{Path, PathBuf};

    // The contents of the generated files should be tested, but there isn't
    // really a need to expose these publicly from the crate
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
        dir.read_dir()
            .unwrap()
            .map(|e| e.unwrap())
            .filter(|entry| entry.file_name() != OsString::from(".gitkeep"))
            .collect()
    }

    pub fn assert_empty_directory(dir: &Path) {
        assert!(dir.is_dir(), "{} should be a directory", dir.display(),);

        let entries = dir_entries(&dir);
        assert_eq!(entries.len(), 0);
    }

    pub fn assert_file_contents_match(path: &Path, contents: &str) {
        assert_eq!(read_to_string(&path).unwrap(), contents,)
    }

    pub fn assert_expected_structure(dir: &Path) {
        let entries = dir_entries(&dir);

        for entry in entries {
            let filename = entry.file_name().into_string().unwrap();
            let path = entry.path();

            match filename.as_str() {
                CONF => {
                    assert_file_contents_match(&path, CONF_TEMPLATE);
                }
                ENV => {
                    assert_file_contents_match(&path, ENV_TEMPLATE);
                }
                ENV_EX => {
                    assert_file_contents_match(&path, ENV_EX_TEMPLATE);
                }
                "revisions" => {
                    assert_empty_directory(&entry.path());
                }
                _ => {}
            }
        }
    }

    pub fn clean_directory(dir: &Path, remove_revisions_dir: bool) {
        for entry in dir_entries(dir) {
            let filename = entry.file_name().into_string().unwrap();
            let path = entry.path();

            let result = match filename.as_str() {
                CONF | ENV | ENV_EX => remove_file(&path),
                "revisions" => {
                    if remove_revisions_dir {
                        remove_dir_all(&path)
                    } else {
                        Ok(())
                    }
                }
                _ => Ok(()),
            };

            result.expect(&format!("Failed to remove {}", path.display()))
        }
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
    assert_eq!(dir_entries(&dir).len(), 4);

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
    assert_eq!(dir_entries(&dir).len(), 4);

    clean_directory(&dir, true);
    assert_empty_directory(&dir);
}

#[test]
fn existing_directory_with_empty_revisions_directory_works() {
    let dir = test_dir("02-empty-revisions");
    let mut entries = dir_entries(&dir);

    assert_eq!(entries.len(), 1);

    let entry = entries.pop().unwrap();

    assert_eq!(entry.file_name(), OsString::from("revisions"));
    assert_empty_directory(&entry.path());

    begin(&dir).unwrap();
    assert_expected_structure(&dir);
    assert_eq!(dir_entries(&dir).len(), 4);

    clean_directory(&dir, false);
}

#[test]
fn existing_directory_with_nonempty_revisions_directory_fails() {
    let dir = test_dir("03-nonempty-revisions");

    // crate error type is a mess and can't implement PartialEq
    match begin(&dir) {
        Err(Error::PathNotEmptyDirectory(path)) => {
            assert_eq!("tests/fixtures/dirs/03-nonempty-revisions/revisions", path,)
        }
        res => panic!("unexpected result {:#?}", res),
    }
}

#[test]
fn existing_nonempty_directory_works() {
    let dir = test_dir("04-nonempty");

    let entry_names = || {
        let mut names = dir_entries(&dir)
            .iter()
            .map(|e| e.file_name().into_string().unwrap())
            .collect::<Vec<String>>();

        names.sort();
        names
    };

    assert_eq!(entry_names(), vec!["another-file.toml", "some-file.json",]);

    begin(&dir).unwrap();
    assert_expected_structure(&dir);
    assert_eq!(dir_entries(&dir).len(), 6);

    clean_directory(&dir, true);

    assert_eq!(entry_names(), vec!["another-file.toml", "some-file.json",]);
}

#[test]
fn existing_nonempty_directory_with_existing_config_fails() {
    let dir = test_dir("05-existing-config");

    match begin(&dir) {
        Err(Error::PathAlreadyExists(path)) => {
            assert_eq!("tests/fixtures/dirs/05-existing-config/jrny.toml", path,)
        }
        res => panic!("unexpected result {:#?}", res),
    }
}

#[test]
fn existing_nonempty_directory_with_existing_env_fails() {
    let dir = test_dir("06-existing-env");

    match begin(&dir) {
        Err(Error::PathAlreadyExists(path)) => {
            assert_eq!("tests/fixtures/dirs/06-existing-env/jrny-env.toml", path,)
        }
        res => panic!("unexpected result {:#?}", res),
    }
}
