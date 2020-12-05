use chrono::{DateTime, TimeZone, Utc};
use sha2::{Digest, Sha256};
use std::cmp::Ordering;
use std::convert::TryFrom;
use std::fs;
use std::path::PathBuf;

/// Metadata and contents for a revision loaded from disk.
#[derive(Debug)]
pub struct RevisionFile {
    pub checksum: String,
    pub contents: String,
    pub created_at: DateTime<Utc>,
    /// The full name of the file, including timestamp and extension
    pub filename: String,
    /// The name of the file, excluding timestamp and extension
    pub name: String,
}

impl RevisionFile {
    /// Attempts to read revision directory to convert all entries (assumed to be SQL files)
    /// into metadata objects with contents stored.
    pub fn all_from_disk(revisions: &PathBuf) -> Result<Vec<Self>, String> {
        let mut entries = fs::read_dir(revisions.as_path())
            .map_err(|e| e.to_string())?
            .map(|res| res.map(|e| e.path()).map_err(|e| e.to_string()))
            .collect::<Result<Vec<_>, String>>()?;

        entries.sort();
        entries.iter().map(Self::try_from).collect()
    }
}

impl TryFrom<&PathBuf> for RevisionFile {
    type Error = String;

    /// Attempts to gather appropriate metadata for and read contents of given path buf.
    fn try_from(p: &PathBuf) -> Result<Self, Self::Error> {
        let filename = p
            .file_name()
            .map(|os_str| os_str.to_str())
            .flatten()
            .ok_or_else(|| format!("{} is not a valid file", p.display()))?;

        let title = RevisionTitle::try_from(filename)?;

        let contents = fs::read_to_string(p)
            .map_err(|e| format!("Could not open {}: {}", p.display(), e.to_string()))?;

        Ok(Self {
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
    pub applied_on: DateTime<Utc>,
    pub checksum: String,
    pub created_at: DateTime<Utc>,
    /// The full name of the file, including timestamp and extension
    pub filename: String,
    /// The name of the file, excluding timestamp and extension
    pub name: String,
}

/// Comprehensive metadata for a revision detected on disk or in the database.
#[derive(Debug, Eq)]
pub struct AnnotatedRevision {
    pub applied_on: Option<DateTime<Utc>>,
    pub checksum: Option<String>,
    pub checksums_match: Option<bool>,
    pub contents: Option<String>,
    pub created_at: DateTime<Utc>,
    /// The full name of the file, including timestamp and extension
    pub filename: String,
    /// The name of the file, excluding timestamp and extension
    pub name: String,
    pub on_disk: bool,
}

impl Ord for AnnotatedRevision {
    fn cmp(&self, other: &Self) -> Ordering {
        (&self.created_at, &self.filename).cmp(&(&other.created_at, &other.filename))
    }
}

impl PartialOrd for AnnotatedRevision {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for AnnotatedRevision {
    fn eq(&self, other: &Self) -> bool {
        self.created_at == other.created_at && self.filename == other.filename
    }
}

/// Creation date and extension-less name extracted from a revision filename.
#[derive(Debug, PartialEq)]
struct RevisionTitle {
    created_at: DateTime<Utc>,
    name: String,
}

impl TryFrom<&str> for RevisionTitle {
    type Error = String;

    fn try_from(filename: &str) -> Result<Self, Self::Error> {
        let err = || {
            format!(
                "Invalid revision name `{}`: expected [timestamp].[name].sql",
                filename,
            )
        };

        let parts: Vec<&str> = filename.split('.').collect();

        // TODO maybe just use nightly?
        // Regex would work too, but it's not worth the 1.2 Mb increase
        /*
        let (timestamp, name) = match parts.as_slice() {
            [timestamp, .. name, "sql"] => (timestamp, name.join(".")),
            _ => return Err(err()),
        };
        */

        if parts.len() < 3 || parts[parts.len() - 1] != "sql" {
            return Err(err());
        }

        let name = parts[1..parts.len() - 1].join(".");
        let timestamp: i64 = parts[0].parse().map_err(|_| err())?;
        let created_at = Utc.timestamp(timestamp, 0);

        Ok(Self {
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
            RevisionTitle::try_from("1577836800.some-file.sql").unwrap(),
            RevisionTitle {
                created_at: Utc.ymd(2020, 1, 1).and_hms(0, 0, 0),
                name: "some-file".to_string(),
            }
        )
    }

    #[test]
    fn revision_title_fails_non_sql() {
        assert_eq!(
            RevisionTitle::try_from("1577836800.some-file.wat"),
            Err(
                "Invalid revision name `1577836800.some-file.wat`: expected [timestamp].[name].sql"
                    .to_string()
            )
        )
    }

    #[test]
    fn revision_title_allows_multiple_periods() {
        assert_eq!(
            RevisionTitle::try_from("1577836800.some.file.sql").unwrap(),
            RevisionTitle {
                created_at: Utc.ymd(2020, 1, 1).and_hms(0, 0, 0),
                name: "some.file".to_string(),
            }
        )
    }
}
