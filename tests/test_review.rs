use std::env;
use std::io::ErrorKind as IoErrorKind;
use std::path::PathBuf;
use jrny::{Config, Environment, Error, review};

mod common;
use common::fs::test_dir;


fn test_cfg(path: impl Into<PathBuf>) -> Config {
    let path = path.into();
    Config::build().revision_directory(&path).finish()
}

fn test_env() -> Environment {
    let url = env::var("JRNY_TEST_DB_URL").unwrap();
    Environment::from_database_url(&url)
}

#[test]
fn bad_env_url_fails() {
    let cfg = test_cfg("doesnt/matter");
    let env = Environment::from_database_url("some-string-doesn't-matter");

    match review(&cfg, &env) {
        Err(Error::DatabaseError(e)) => {
            assert_eq!(
                "invalid connection string: unexpected EOF",
                &format!("{}", e),
            )
        }
        res => panic!("unexpected result {:#?}", res),
    }
}

#[test]
fn wrong_env_url_fails() {
    let cfg = test_cfg("doesnt/matter");
    let env = Environment::from_database_url("postgresql://fakeuser:fakepassword@localhost:9999/please_be_fake");

    match review(&cfg, &env) {
        Err(Error::DatabaseError(e)) => {
            assert_eq!(
                "error connecting to server: Connection refused (os error 61)",
                &format!("{}", e),
            )
        }
        res => panic!("unexpected result {:#?}", res),
    }
}

#[test]
fn missing_cfg_revisions_dir_fails() {
    let cfg = test_cfg(test_dir("review/01-empty-dir/revisions"));
    let env = test_env();

    match review(&cfg, &env) {
        Err(Error::IoError(e)) => {
            assert_eq!(e.kind(), IoErrorKind::NotFound);
        }
        res => panic!("unexpected result {:#?}", res),
    }
}

#[test]
fn duplicate_ids_fails() {
    let cfg = test_cfg(test_dir("review/03-duplicate-ids/revisions"));
    let env = test_env();

    match review(&cfg, &env) {
        Err(Error::RevisionsFailedReview(summary)) => {
            assert_eq!(summary.duplicate_ids(), 2);
            assert_eq!(summary.files_changed(), 0);
            assert_eq!(summary.files_not_found(), 0);
            assert_eq!(summary.preceding_applied(), 0);
        }
        res => panic!("unexpected result {:#?}", res),
    }
}

#[test]
fn file_changed_fails() {}

#[test]
fn file_not_found_fails() {}

#[test]
fn pending_before_applied_fails() {}

#[test]
fn all_errors_fails() {}

#[test]
fn no_revisions_works() {
    let cfg = test_cfg(test_dir("review/02-empty-revisions/revisions"));
    let env = test_env();

    review(&cfg, &env).unwrap();
}

#[test]
fn only_pending_works() {
    let cfg = test_cfg(test_dir("review/04-only-pending/revisions"));
    let env = test_env();

    review(&cfg, &env).unwrap();
}

#[test]
fn mix_of_pending_and_applied_works() {}