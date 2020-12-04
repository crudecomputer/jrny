use chrono::Utc;
use std::{fs, io::Write, path::PathBuf};

use crate::config::Config;

pub struct Plan {
    pub filename: String,
    path: PathBuf,
}

impl Plan {
    pub fn new_revision(config: &Config, name: &str) -> Result<Self, String> {
        let timestamp = Utc::now().timestamp();

        let revision_path = config
            .paths
            .revisions
            .join(format!("{}.{}.sql", timestamp, name));

        let cmd = Self {
            filename: revision_path.display().to_string(),
            path: revision_path,
        };

        Ok(cmd.create_file()?)
    }

    fn create_file(self) -> Result<Self, String> {
        fs::File::create(&self.path)
            .map_err(|e| e.to_string())?
            .write_all(format!("-- Journey revision\n--\n-- {}\n--\n\n", self.filename,).as_bytes())
            .map_err(|e| e.to_string())?;

        Ok(self)
    }
}
