use chrono::{DateTime, TimeZone, Utc};
use sha2::{Digest, Sha256};
use std::convert::TryFrom;
use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
pub struct DatabaseRevision {
    pub applied_on: DateTime<Utc>,
    pub checksum: String,
    pub filename: String,
    pub on_disk: Option<bool>,
    //pub name: String,
    //pub timestamp: DateTime<Utc>,
}

#[derive(Debug)]
pub struct FileRevision {
    pub applied: Option<bool>,
    pub checksum: String,
    pub contents: String,
    pub filename: String,
    //pub name: String,
    //pub timestamp: DateTime<Utc>,
}

impl FileRevision {
    pub fn all_from_disk(revisions: &PathBuf) -> Result<Vec<Self>, String> {
        // Assumes all files in directory are revisions. This isn't strictly
        // necessary but there's also not likely a reason for having other
        // files in there alongside the SQL revisions.
        let mut entries = fs::read_dir(revisions.as_path())
            .map_err(|e| e.to_string())?
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

impl TryFrom<&PathBuf> for FileRevision {
    type Error = String;

    fn try_from(p: &PathBuf) -> Result<Self, Self::Error> {
        let filename = p.file_stem()
            .map(|os_str| os_str.to_str())
            .flatten()
            .ok_or_else(|| format!("{} is not a valid file", p.display()))?;

        //let parts: Vec<&str> = filename.splitn(2, ".").collect();

        //let err = || format!(
            //"Invalid revision name {}: expected <timestamp>.<name>.sql",
            //filename,
        //);

        //let (seconds, name) = match (parts.get(0), parts.get(1)) {
            //(Some(seconds), Some(name)) => (seconds, name),
            //_ => return Err(err()),
        //};
        //let seconds: i64 = seconds.parse().map_err(|_| err())?;
        //let timestamp = Utc.timestamp(seconds, 0);

        let contents = fs::read_to_string(p)
            .map_err(|e| format!(
                "Could not open {}: {}",
                p.display(),
                e.to_string()
            ))?;

        Ok(Self {
            applied: None,
            checksum: to_checksum(&contents),
            contents,
            filename: filename.to_string(),
            //name: name.to_string(),
            //timestamp,
        })
    }
}

fn to_checksum(s: &str) -> String {
    // See: https://users.rust-lang.org/t/sha256-result-to-string/49391/3
    format!("{:x}", Sha256::digest(s.as_bytes()))
}
