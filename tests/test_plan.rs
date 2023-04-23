use std::fs::remove_file;
use std::io::ErrorKind;
use std::path::Path;
use std::ops::Range;
use std::thread::sleep;
use std::time::Duration;

use chrono::Utc;
use jrny::{Config, Error, plan};

mod common;
use common::fs::{
    assert_empty_directory,
    assert_file_contents_match,
    test_dir,
};

const DEFAULT_CONTENTS: &str = "-- Revision: {title}
--
-- Add description here

begin;

-- Add SQL here

commit;
";

fn assert_revision(
    dir: &Path,
    id: &str,
    timestamp_range: Range<i64>,
    name: &str,
    expected_contents: &str,
) {
    assert_file_contents_match(dir, expected_contents);

    let filename = dir.file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    let parts = filename
        .split(".")
        .collect::<Vec<_>>();

    assert!(parts.len() >= 4);

    let timestamp = parts[1].parse::<i64>().unwrap();

    assert_eq!(parts[0], id);
    assert_eq!(parts[parts.len() - 1], "sql");

    let actual_name = parts[2..parts.len() - 1].join(".");
    assert_eq!(actual_name, name);

    assert!(timestamp_range.contains(&timestamp));
}

#[test]
fn invalid_project_config_fails() {
    let dir = test_dir("plan/01-no-revisions-dir/revisions");
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
    let title = "a-revision-file";

    let dir = test_dir("plan/02a-empty-revisions-dir/revisions");
    let cfg = Config::build()
        .revision_directory(&dir)
        .finish();

    assert_empty_directory(&dir);

    let start = Utc::now().timestamp();
    sleep(Duration::from_secs(1));

    let created = plan(&cfg, title, None).unwrap();

    sleep(Duration::from_secs(1));
    let end = Utc::now().timestamp();

    assert_revision(
        &created,
        "001",
        start..end,
        title,
        &DEFAULT_CONTENTS.replace("{title}", title),
    );
    remove_file(&created).unwrap();
}


#[test]
fn no_existing_revisions_and_pass_contents() {
    let title = "different-revision-file";

    let dir = test_dir("plan/02b-empty-revisions-dir/revisions");
    let cfg = Config::build()
        .revision_directory(&dir)
        .finish();

    assert_empty_directory(&dir);

    let start = Utc::now().timestamp();
    sleep(Duration::from_secs(1));

    let created = plan(
        &cfg,
        title,
        Some("drop database haha;"),
    ).unwrap();

    sleep(Duration::from_secs(1));
    let end = Utc::now().timestamp();

    assert_revision(
        &created,
        "001",
        start..end,
        title,
        "drop database haha;",
    );
    remove_file(&created).unwrap();
}

#[test]
fn existing_revisions() {
    let title = "another.revision.file";

    let dir = test_dir("plan/03a-existing-revisions/revisions");
    let cfg = Config::build()
        .revision_directory(&dir)
        .finish();

    let start = Utc::now().timestamp();
    sleep(Duration::from_secs(1));

    let created = plan(&cfg, title, None).unwrap();

    sleep(Duration::from_secs(1));
    let end = Utc::now().timestamp();

    assert_revision(
        &created,
        "004",
        start..end,
        title,
        &DEFAULT_CONTENTS.replace("{title}", title),
    );
    remove_file(&created).unwrap();
}

#[test]
fn existing_revisions_with_gap_and_pass_contents() {
    let title = "another revision file";

    let dir = test_dir("plan/03b-existing-revisions/revisions");
    let cfg = Config::build()
        .revision_directory(&dir)
        .finish();

    let start = Utc::now().timestamp();
    sleep(Duration::from_secs(1));

    let created = plan(&cfg, title, None).unwrap();

    sleep(Duration::from_secs(1));
    let end = Utc::now().timestamp();

    assert_revision(
        &created,
        "005",
        start..end,
        title,
        &DEFAULT_CONTENTS.replace("{title}", title),
    );
    remove_file(&created).unwrap();
}
