use chrono::Utc;
use std::{fs, io::Write, path::PathBuf};

use crate::{Result, config::Config};

pub struct Plan {
    pub filename: String,
    path: PathBuf,
}

impl Plan {
    pub fn new_revision(config: &Config, name: &str) -> Result<Self> {
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

    fn create_file(self) -> Result<Self> {
        fs::File::create(&self.path)?
            .write_all(format!(
                "-- Journey revision\n--\n-- {}\n--\n\n",
                self.filename,).as_bytes()
            )?;

        Ok(self)
    }
}
