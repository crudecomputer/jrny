use chrono::Utc;
use std::{fs, io::Write, path::PathBuf};

use crate::{config::Config, Result};

const TEMPLATE: &str = "-- :filename

begin;
-- Start revisions


-- End revisions
commit;
";

pub struct Plan {
    pub filename: String,
    path: PathBuf,
}

impl Plan {
    pub fn new_revision(config: &Config, name: &str) -> Result<Self> {
        let timestamp = Utc::now().timestamp();

        let filename = format!("{:03}.{}.{}.sql", config.next_id, timestamp, name);
        let revision_path = config.paths.revisions.join(&filename);

        let cmd = Self {
            filename,
            path: revision_path,
        };

        Ok(cmd.create_file()?)
    }

    fn create_file(self) -> Result<Self> {
        fs::File::create(&self.path)?
            .write_all(TEMPLATE.replace(":filename", &self.filename).as_bytes())?;

        Ok(self)
    }
}
