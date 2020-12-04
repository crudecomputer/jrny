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
struct RevisionTitle {
    created_at: DateTime<Utc>,
    name: String,
}

impl TryFrom<&str> for RevisionTitle {
    type Error = String;

    fn try_from(filename: &str) -> Result<Self, Self::Error> {
        // Regex would work great here, but not sure if it's worth the 1.2 Mb increase
        // in binary size, especially since (I believe) unicode tables would be necessary
        // and that's the obvious feature to disable to reduce size
        let parts: Vec<&str> = filename.splitn(3, '.').collect();

        let err = || {
            format!(
                "Invalid revision name {}: expected <timestamp>.<name>.sql",
                filename,
            )
        };

        let (timestamp, name) = match (parts.get(0), parts.get(1), parts.get(2)) {
            (Some(seconds), Some(name), Some(&"sql")) => (seconds, name),
            _ => return Err(err()),
        };
        let timestamp: i64 = timestamp.parse().map_err(|_| err())?;
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
    #[test]
    fn parses_filename() {
        // test name allowed to have "." in it
        assert_eq!(true, false)
    }
}
