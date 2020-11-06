use postgres::Client;
use std::convert::TryFrom;

use crate::Config;

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
    client: Client,
    schema: String,
    table: String,
}

impl Executor {
    pub fn new(config: &Config) -> Result<Self, String> {
        let client = Client::try_from(config)?;

        Ok(Self {
            client,
            schema: config.settings.table.schema.clone(),
            table: config.settings.table.name.clone(),
        })
    }

    pub fn ensure_table_exists(&mut self) -> Result<(), String> {
        if !self.schema_exists()? { self.create_schema()?; }
        if !self.table_exists()? { self.create_table()?; }

        Ok(())
    }

    fn table_exists(&mut self) -> Result<bool, String> {
        let row = self.client.query_one(TABLE_EXISTS, &[
            &self.schema,
            &self.table,
        ]).map_err(|e| e.to_string())?;

        Ok(row.get("exists"))
    }

    fn schema_exists(&mut self) -> Result<bool, String> {
        let row = self.client.query_one(SCHEMA_EXISTS, &[
            &self.schema,
        ]).map_err(|e| e.to_string())?;

        Ok(row.get("exists"))
    }

    fn create_schema(&mut self) -> Result<(), String> {
        println!("Creating schema {}", self.schema);
        let create = CREATE_SCHEMA.replace("$$schema$$", &self.schema);

        self.client.execute(create.as_str(), &[]).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn create_table(&mut self) -> Result<(), String> {
        println!("Creating table {}.{}", self.schema, self.table);
        let create = CREATE_TABLE
            .replace("$$schema$$", &self.schema)
            .replace("$$table$$", &self.table);

        self.client.execute(create.as_str(), &[]).map_err(|e| e.to_string())?;
        Ok(())
    }
}
