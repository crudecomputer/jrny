use std::io::ErrorKind;
use jrny::{Config, Error, plan};

mod common;
use common::fs::{test_dir};

#[test]
fn invalid_project_config_fails() {
    let dir = test_dir("05-existing-config/revisions");
    let cfg = Config::build()
        .revision_directory(&dir)
        .finish();

    match plan(&cfg, "some-revision", None) {
        Err(Error::IoError(e)) => {
            match e.kind() {
                ErrorKind::NotFound => {},
                _ => panic!("unexpected error kind {:#?}", e)
            }
        },
        res => panic!("unexpected result {:#?}", res),
    }
}

#[test]
fn no_existing_revisions() {
}

#[test]
fn no_existing_revisions_and_pass_contents() {
}

#[test]
fn existing_revisions() {
}

#[test]
fn existing_revisions_and_pass_contents() {
}

#[test]
fn existing_revisions_with_gap() {
}
