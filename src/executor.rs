use postgres::Client;
use std::convert::TryFrom;

use crate::config::Config;

const EXISTS: &str =
"SELECT EXISTS (
   SELECT FROM pg_tables
   WHERE schemaname = $1 AND tablename  = $2
)";

const CREATE: &str =
"CREATE TABLE $$schema$$.$$table$$ (
    id          SERIAL       PRIMARY KEY,
    timestamp   TIMESTAMPTZ  NOT NULL,
    applied_on  TIMESTAMPTZ  NOT NULL,
    filename    TEXT         NOT NULL,
    checksum    TEXT         NOT NULL,

    unique (timestamp, filename)
)";

pub struct Executor {
    config: Config,
    client: Client,
}

impl Executor {
    pub fn new(conf_path_name: Option<&str>) -> Result<Self, String> {
        let config = Config::new(conf_path_name)?;
        let client = Client::try_from(&config)?;

        Ok(Self { config , client })
    }

    pub fn ensure_table_exists(&mut self) -> Result<(), String> {
        if !self.table_exists()? {
            println!("Creating table \"{}\"", self.qualified_table());
            self.create_table()?;
        }

        Ok(())
    }

    fn table_exists(&mut self) -> Result<bool, String> {
        let row = self.client.query_one(EXISTS, &[
            &self.config.app.schema, 
            &self.config.app.table, 
        ]).map_err(|e| e.to_string())?;

        Ok(row.get("exists"))
    }

    fn create_table(&mut self) -> Result<(), String> {
        let create = CREATE
            .replace("$$schema$$", &self.config.app.schema)
            .replace("$$table$$", &self.config.app.table);

        self.client.execute(create.as_str(), &[]).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn qualified_table(&self) -> String {
        format!("{}.{}", &self.config.app.schema.clone(), self.config.app.table.clone())
    }
}
