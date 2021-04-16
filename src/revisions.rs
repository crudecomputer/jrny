use chrono::{DateTime, TimeZone, Utc};
use sha2::{Digest, Sha256};
use std::cmp::Ordering;
use std::convert::TryFrom;
use std::fs;
use std::path::PathBuf;

use crate::{Error, Result};

/// Metadata and contents for a revision loaded from disk.
#[derive(Debug)]
pub struct RevisionFile {
    /// The file id of the revision
    pub id: i32,
    /// The hash of the contents
    pub checksum: String,
    /// Contents of revision file, if present
    pub contents: String,
    /// Moment the revision was created
    pub created_at: DateTime<Utc>,
    /// The full name of the file, including id, timestamp, and extension
    pub filename: String,
    /// The name of the file, excluding id, timestamp, and extension
    pub name: String,
}

impl RevisionFile {
    /// Attempts to read revision directory to convert all entries (assumed to be SQL files)
    /// into metadata objects with contents stored.
    pub fn all_from_disk(revisions: &PathBuf) -> Result<Vec<Self>> {
        let mut entries = fs::read_dir(revisions.as_path())?
            .map(|res| res.map(|e| e.path()).map_err(Error::IoError))
            .collect::<Result<Vec<_>>>()?;

        entries.sort();
        entries.iter().map(Self::try_from).collect()
    }
}

impl TryFrom<&PathBuf> for RevisionFile {
    type Error = crate::Error;

    /// Attempts to gather appropriate metadata for and read contents of given path buf.
    fn try_from(p: &PathBuf) -> std::result::Result<Self, Self::Error> {
        let filename = p
            .file_name()
            .map(|os_str| os_str.to_str())
            .flatten()
            .ok_or_else(|| Error::FileNotValid(p.display().to_string()))?;

        let title = RevisionTitle::try_from(filename)?;
        let contents = fs::read_to_string(p)?;

        Ok(Self {
            id: title.id,
            checksum: to_checksum(&contents),
            contents,
            created_at: title.created_at,
            filename: filename.to_string(),
            name: title.name,
        })
    }
}

/// Metadata stored for a revision that has already been applied.
#[derive(Debug)]
pub struct RevisionRecord {
    /// The database id of the revision
    pub id: i32,
    /// Moment the revision was applied to the database
    pub applied_on: DateTime<Utc>,
    /// The hash of the contents
    pub checksum: String,
    /// Moment the revision was created
    pub created_at: DateTime<Utc>,
    /// The full name of the file, including timestamp and extension
    pub filename: String,
    /// The name of the file, excluding timestamp and extension
    pub name: String,
}

/// Comprehensive metadata for a revision detected on disk or in the database.
#[derive(Debug, PartialEq, Eq)]
pub struct AnnotatedRevision {
    /// The id of both the record and the revision file
    pub id: i32,
    /// Moment the revision was applied to the database
    pub applied_on: Option<DateTime<Utc>>,
    /// Checksum of the revision file on disk, if present
    pub checksum: Option<String>,
    /// Whether or not checksums for file and record match, if both are present
    pub checksums_match: Option<bool>,
    /// Contents of revision file, if present
    pub contents: Option<String>,
    /// Moment the revision was created
    pub created_at: DateTime<Utc>,
    /// The full name of the file, including id, timestamp, and extension
    pub filename: String,
    /// The name of the file, excluding id, timestamp, and extension
    pub name: String,
    /// Whether or not the file for an applied revision is found on disk
    pub on_disk: bool,
}

impl Ord for AnnotatedRevision {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

impl PartialOrd for AnnotatedRevision {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Creation date and extension-less name extracted from a revision filename.
#[derive(Debug, PartialEq)]
struct RevisionTitle {
    /// The numeric id extracted from the filename
    id: i32,
    /// The file creation moment extracted from the filename
    created_at: DateTime<Utc>,
    /// The remaining portion of the filename, excluding extension
    name: String,
}

impl TryFrom<&str> for RevisionTitle {
    type Error = crate::Error;

    fn try_from(filename: &str) -> std::result::Result<Self, Self::Error> {
        let parts: Vec<&str> = filename.split('.').collect();

        // Regex would work too, but not sure it's worth the dependencies and
        // binary size increase.
        //
        // TODO maybe just use nightly? Eg.
        //
        //     let (timestamp, name) = match parts.as_slice() {
        //         [timestamp, .. name, "sql"] => (timestamp, name.join(".")),
        //         _ => return Err(err()),
        //     };

        if parts.len() < 4 || parts.last() != Some(&"sql") {
            return Err(Error::RevisionNameInvalid(filename.to_string()));
        }

        let id: i32 = parts[0].parse()
            .map_err(|_| Error::RevisionNameInvalid(filename.to_string()))?;

        let timestamp: i64 = parts[1]
            .parse()
            .map_err(|e| Error::RevisionTimestampInvalid(e, filename.to_string()))?;

        // Utc.timestamp can panic, hence `timestamp_opt`
        let created_at = Utc
            .timestamp_opt(timestamp, 0)
            .single()
            .ok_or_else(|| Error::RevisionTimestampOutOfRange(filename.to_string()))?;

        let name = parts[2..parts.len() - 1].join(".");

        Ok(Self {
            id,
            created_at,
            name: (*name).to_string(),
        })
    }
}

fn to_checksum(s: &str) -> String {
    // See: https://users.rust-lang.org/t/sha256-result-to-string/49391/3
    format!("{:x}", Sha256::digest(s.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn revision_title_parses_sql_filename() {
        assert_eq!(
            RevisionTitle::try_from("001.1577836800.some-file.sql").unwrap(),
            RevisionTitle {
                id: 1,
                created_at: Utc.ymd(2020, 1, 1).and_hms(0, 0, 0),
                name: "some-file".to_string(),
            }
        )
    }

    #[test]
    fn revision_title_allows_multiple_periods() {
        assert_eq!(
            RevisionTitle::try_from("003.1577836800.some.file.sql").unwrap(),
            RevisionTitle {
                id: 3,
                created_at: Utc.ymd(2020, 1, 1).and_hms(0, 0, 0),
                name: "some.file".to_string(),
            }
        )
    }

    #[test]
    fn revision_title_fails_non_sql() {
        match RevisionTitle::try_from("001.1577836800.some-file.wat") {
            Err(Error::RevisionNameInvalid(filename)) => {
                assert_eq!(filename, "001.1577836800.some-file.wat".to_string());
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn revision_title_fails_bad_timestamp() {
        match RevisionTitle::try_from("001.asdf.some-file.sql") {
            Err(Error::RevisionTimestampInvalid(_, filename)) => {
                assert_eq!(filename, "001.asdf.some-file.sql".to_string());
            }
            result => panic!("received {:?}", result),
        }
        match RevisionTitle::try_from("001.9999999999999999.some-file.sql") {
            Err(Error::RevisionTimestampOutOfRange(filename)) => {
                assert_eq!(filename, "001.9999999999999999.some-file.sql".to_string());
            }
            result => panic!("received {:?}", result),
        }
    }
}
