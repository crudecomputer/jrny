use chrono::{DateTime, TimeZone, Utc};
use sha2::{Digest, Sha256};
use std::cmp::Ordering;
use std::convert::TryFrom;
use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
pub struct RevisionFile {
    pub checksum: String,
    pub contents: String,
    pub created_at: DateTime<Utc>,
    pub filename: String,
    pub name: String,
}

impl RevisionFile {
    pub fn all_from_disk(revisions: &PathBuf) -> Result<Vec<Self>, String> {
        // Assumes all files in directory are revisions. This isn't strictly
        // necessary but there's also not likely a reason for having other
        // files in there alongside the SQL revisions.
        let mut entries = fs::read_dir(revisions.as_path())
            .map_err(|e| e.to_string())?
            // TODO filter to .sql files
            .map(|res| res
                 .map(|e| e.path())
                 .map_err(|e| e.to_string())
            )
            .collect::<Result<Vec<_>, String>>()?;

        // All entries should be prefixed with timestamp for easy sorting
        entries.sort();
        entries.iter().map(Self::try_from).collect()
    }
}

impl TryFrom<&PathBuf> for RevisionFile {
    type Error = String;

    fn try_from(p: &PathBuf) -> Result<Self, Self::Error> {
        let filename = p.file_name()
            .map(|os_str| os_str.to_str())
            .flatten()
            .ok_or_else(|| format!("{} is not a valid file", p.display()))?;

        // Regex would work great here, but not sure if it's worth the 1.2 Mb increase
        // in binary size, especially since (I believe) unicode tables would be necessary
        // and that's the obvious feature to disable to reduce size
        let parts: Vec<&str> = filename.splitn(3, ".").collect();

        let err = || format!(
            "Invalid revision name {}: expected <timestamp>.<name>.sql",
            filename,
        );

        let (seconds, name) = match (parts.get(0), parts.get(1), parts.get(2)) {
            (Some(seconds), Some(name), Some(&"sql")) => (seconds, name),
            _ => return Err(err()),
        };
        let seconds: i64 = seconds.parse().map_err(|_| err())?;
        let created_at = Utc.timestamp(seconds, 0);

        let contents = fs::read_to_string(p)
            .map_err(|e| format!(
                "Could not open {}: {}",
                p.display(),
                e.to_string()
            ))?;

        Ok(Self {
            checksum: to_checksum(&contents),
            contents,
            created_at,
            filename: filename.to_string(),
            name: name.to_string(),
        })
    }
}

#[derive(Debug)]
pub struct RevisionRecord {
    pub applied_on: DateTime<Utc>,
    pub checksum: String,
    pub filename: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Eq)]
pub struct AnnotatedRevision {
    pub applied_on: Option<DateTime<Utc>>,
    pub checksum: Option<String>,
    pub checksums_match: Option<bool>,
    pub contents: Option<String>,
    pub created_at: DateTime<Utc>,
    pub filename: String,
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

fn to_checksum(s: &str) -> String {
    // See: https://users.rust-lang.org/t/sha256-result-to-string/49391/3
    format!("{:x}", Sha256::digest(s.as_bytes()))
}
