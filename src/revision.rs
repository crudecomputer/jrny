use std::fs::{self, DirEntry};
use std::path::PathBuf;

use crate::Config;

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
            checksum: "asdjhlakjsdhflkaj".to_string(),
            name: "name".to_string(),
            timestamp: "timestamp".to_string(),
        })
    }
}
