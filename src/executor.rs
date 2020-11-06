use postgres::Client;
use std::convert::TryFrom;

use crate::config::Config;

const CREATE_SCHEMA: &str =
"CREATE SCHEMA $$schema$$";

const CREATE_TABLE: &str =
"CREATE TABLE $$schema$$.$$table$$ (
    id          SERIAL       PRIMARY KEY,
    timestamp   TIMESTAMPTZ  NOT NULL,
    applied_on  TIMESTAMPTZ  NOT NULL,
    filename    TEXT         NOT NULL,
    checksum    TEXT         NOT NULL,

    UNIQUE (timestamp, filename)
)";

const TABLE_EXISTS: &str =
"SELECT EXISTS (
   SELECT FROM pg_tables
   WHERE schemaname = $1 AND tablename  = $2
)";

const SCHEMA_EXISTS: &str =
"SELECT EXISTS (
    SELECT FROM information_schema.schemata
    WHERE schema_name = $1
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
        if !self.schema_exists()? { self.create_schema()?; }
        if !self.table_exists()? { self.create_table()?; }

        Ok(())
    }

    fn table_exists(&mut self) -> Result<bool, String> {
        let row = self.client.query_one(TABLE_EXISTS, &[
            &self.config.settings.table.schema,
            &self.config.settings.table.name,
        ]).map_err(|e| e.to_string())?;

        Ok(row.get("exists"))
    }

    fn schema_exists(&mut self) -> Result<bool, String> {
        let row = self.client.query_one(SCHEMA_EXISTS, &[
            &self.config.settings.table.schema,
        ]).map_err(|e| e.to_string())?;

        Ok(row.get("exists"))
    }

    fn create_schema(&mut self) -> Result<(), String> {
        println!("Creating schema {}", self.config.settings.table.schema);
        let create = CREATE_SCHEMA
            .replace("$$schema$$", &self.config.settings.table.schema);

        self.client.execute(create.as_str(), &[]).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn create_table(&mut self) -> Result<(), String> {
        println!(
            "Creating table {}.{}",
            self.config.settings.table.schema,
            self.config.settings.table.name,
        );
        let create = CREATE_TABLE
            .replace("$$schema$$", &self.config.settings.table.schema)
            .replace("$$table$$", &self.config.settings.table.name);

        self.client.execute(create.as_str(), &[]).map_err(|e| e.to_string())?;
        Ok(())
    }
}
