use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
pub struct FileRevision {
    applied: bool,
    checksum: String,
    name: String,
    timestamp: String,
}

impl FileRevision {
    pub fn all_from_disk(revisions: &PathBuf) -> Result<Vec<Self>, String> {
        let mut entries = fs::read_dir(revisions.as_path())
            .map_err(|e| e.to_string())?
            .map(|res| res.map(|e| e.path()).map_err(|e| e.to_string()))
            .collect::<Result<Vec<_>, String>>()?;

        // All entries should be prefixed with timestamp for easy sorting
        entries.sort();

        entries.iter().map(Self::from_entry).collect()
    }

    fn from_entry(path: &PathBuf) -> Result<Self, String> {
        let contents = fs::read_to_string(path)
            .map_err(|e| format!(
                "Could not open {}: {}",
                path.display(),
                e.to_string()
            ))?;

        Ok(Self {
            applied: false,
            checksum: to_checksum(&contents),
            name: "name".to_string(),
            timestamp: "timestamp".to_string(),
        })
    }
}

fn to_checksum(s: &str) -> String {
    // See: https://users.rust-lang.org/t/sha256-result-to-string/49391/3
    format!("{:x}", Sha256::digest(s.as_bytes()))
}
