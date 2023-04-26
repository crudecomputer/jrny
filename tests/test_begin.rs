use jrny::{begin, Error};
use std::ffi::OsString;
use std::fs::remove_dir_all;

mod common;
use common::fs::*;

#[test]
fn new_project_directory_works() {
    let dir = test_dir("begin/00-nonexistent");

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
    let dir = test_dir("begin/01-existing-empty");

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
    let dir = test_dir("begin/02-empty-revisions");
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
    let dir = test_dir("begin/03-nonempty-revisions");

    // crate error type is a mess and can't implement PartialEq
    match begin(&dir) {
        Err(Error::PathNotEmptyDirectory(path)) => {
            assert_eq!("tests/fixtures/begin/03-nonempty-revisions/revisions", path,)
        }
        res => panic!("unexpected result {:#?}", res),
    }
}

#[test]
fn existing_nonempty_directory_works() {
    let dir = test_dir("begin/04-nonempty");

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
    let dir = test_dir("begin/05-existing-config");

    match begin(&dir) {
        Err(Error::PathAlreadyExists(path)) => {
            assert_eq!("tests/fixtures/begin/05-existing-config/jrny.toml", path,)
        }
        res => panic!("unexpected result {:#?}", res),
    }
}

#[test]
fn existing_nonempty_directory_with_existing_env_fails() {
    let dir = test_dir("begin/06-existing-env");

    match begin(&dir) {
        Err(Error::PathAlreadyExists(path)) => {
            assert_eq!("tests/fixtures/begin/06-existing-env/jrny-env.toml", path,)
        }
        res => panic!("unexpected result {:#?}", res),
    }
}
