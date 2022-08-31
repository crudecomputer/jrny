use std::convert::TryFrom;

use log::info;
use postgres::Client;

use crate::{
    revisions::{AnnotatedRevision, RevisionRecord},
    Config, Environment, Result,
};

const CREATE_SCHEMA: &str = "
CREATE SCHEMA $$schema$$
";

const CREATE_TABLE: &str = "
CREATE TABLE $$schema$$.$$table$$ (
    id          INT          PRIMARY KEY,
    created_at  TIMESTAMPTZ  NOT NULL,
    applied_on  TIMESTAMPTZ  NOT NULL,
    filename    TEXT         NOT NULL UNIQUE,
    name        TEXT         NOT NULL,
    checksum    TEXT         NOT NULL
)";

const TABLE_EXISTS: &str = "
SELECT EXISTS (
   SELECT FROM pg_tables
   WHERE schemaname = $1 AND tablename  = $2
)";

const SCHEMA_EXISTS: &str = "
SELECT EXISTS (
    SELECT FROM information_schema.schemata
    WHERE schema_name = $1
)";

const SELECT_REVISIONS: &str = "
SELECT
    id,
    applied_on,
    checksum,
    created_at,
    filename,
    name
FROM $$schema$$.$$table$$
ORDER BY id ASC
";

const INSERT_REVISION: &str = "
INSERT INTO $$schema$$.$$table$$ (
    applied_on,
    id,
    created_at,
    checksum,
    filename,
    name
) VALUES (clock_timestamp(), $1, $2, $3, $4, $5)
";

pub struct Executor {
    client: Client,
    schema: String,
    table: String,
}

impl Executor {
    pub fn new(config: &Config, env: &Environment) -> Result<Self> {
        let client = Client::try_from(env)?;

        Ok(Self {
            client,
            schema: config.table.schema.clone(),
            table: config.table.name.clone(),
        })
    }

    pub fn ensure_table_exists(&mut self) -> Result<()> {
        if !self.schema_exists()? {
            self.create_schema()?;
        }
        if !self.table_exists()? {
            self.create_table()?;
        }

        Ok(())
    }

    pub fn load_revisions(&mut self) -> Result<Vec<RevisionRecord>> {
        let stmt = SELECT_REVISIONS
            .replace("$$schema$$", &self.schema)
            .replace("$$table$$", &self.table);

        let rows = self.client.query(stmt.as_str(), &[])?;

        let revisions = rows
            .iter()
            .map(|r| RevisionRecord {
                id: r.get("id"),
                applied_on: r.get("applied_on"),
                created_at: r.get("created_at"),
                checksum: r.get("checksum"),
                filename: r.get("filename"),
                name: r.get("name"),
            })
            .collect();

        Ok(revisions)
    }

    pub fn run_revision(&mut self, revision: &AnnotatedRevision) -> Result<()> {
        let insert_revision = INSERT_REVISION
            .replace("$$schema$$", &self.schema)
            .replace("$$table$$", &self.table);

        let statements = revision
            .contents
            .as_ref()
            .expect(format!("No content for {}", revision.filename).as_str());

        let _ = self.client.batch_execute(statements.as_str())?;

        let _ = self.client.execute(
            insert_revision.as_str(),
            &[
                &revision.id,
                &revision.created_at,
                &revision.checksum,
                &revision.filename,
                &revision.name,
            ],
        )?;

        Ok(())
    }

    fn table_exists(&mut self) -> Result<bool> {
        let row = self
            .client
            .query_one(TABLE_EXISTS, &[&self.schema, &self.table])?;

        Ok(row.get("exists"))
    }

    fn schema_exists(&mut self) -> Result<bool> {
        let row = self.client.query_one(SCHEMA_EXISTS, &[&self.schema])?;

        Ok(row.get("exists"))
    }

    fn create_schema(&mut self) -> Result<()> {
        info!("Creating schema {}", self.schema);
        let create = CREATE_SCHEMA.replace("$$schema$$", &self.schema);

        self.client.execute(create.as_str(), &[])?;

        Ok(())
    }

    fn create_table(&mut self) -> Result<()> {
        info!("Creating table {}.{}", self.schema, self.table);
        let create = CREATE_TABLE
            .replace("$$schema$$", &self.schema)
            .replace("$$table$$", &self.table);

        self.client.execute(create.as_str(), &[])?;

        Ok(())
    }
}
