use log::info;
use postgres::Client;
use std::convert::TryFrom;

use crate::config::Config;
use crate::revisions::{AnnotatedRevision, RevisionRecord};
use crate::statements::StatementGroup;

const CREATE_SCHEMA: &str =
"CREATE SCHEMA $$schema$$";

const CREATE_TABLE: &str =
"CREATE TABLE $$schema$$.$$table$$ (
    id          SERIAL       PRIMARY KEY,
    applied_on  TIMESTAMPTZ  NOT NULL,
    created_at  TIMESTAMPTZ  NOT NULL,
    filename    TEXT         NOT NULL UNIQUE,
    name        TEXT         NOT NULL,
    checksum    TEXT         NOT NULL
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

const SELECT_REVISIONS: &str =
"SELECT
    applied_on,
    checksum,
    created_at,
    filename,
    name
FROM $$schema$$.$$table$$
ORDER BY created_at ASC
";

const INSERT_REVISION: &str =
"INSERT INTO $$schema$$.$$table$$ (
    applied_on,
    created_at,
    checksum,
    filename,
    name
) VALUES (now(), $1, $2, $3, $4)
";

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

    pub fn load_revisions(&mut self) -> Result<Vec<RevisionRecord>, String> {
        let stmt = SELECT_REVISIONS
            .replace("$$schema$$", &self.schema)
            .replace("$$table$$", &self.table);

        let rows = self.client.query(stmt.as_str(), &[])
            .map_err(|e| e.to_string())?;

        let revisions = rows.iter()
            .map(|r| RevisionRecord {
                applied_on: r.get("applied_on"),
                checksum: r.get("checksum"),
                created_at: r.get("created_at"),
                filename: r.get("filename"),
                name: r.get("name"),
            })
            .collect();

        Ok(revisions)
    }

    /// Executes all statement groups inside a single transaction, only committing
    /// if explicitly specified.
    pub fn run_revisions(
        &mut self,
        groups: &Vec<(AnnotatedRevision, StatementGroup)>,
        commit: bool,
    ) -> Result<(), String> {
        let insert_revision = INSERT_REVISION
            .replace("$$schema$$", &self.schema)
            .replace("$$table$$", &self.table);

        let mut tx = self.client.transaction()
            .map_err(|e| e.to_string())?;

        for (revision, group) in groups.iter() {
            info!("\nApplying \"{}\"", revision.filename);

            for statement in group.iter() {
                let preview = statement.0.lines()
                    .filter(|l| !l.is_empty())
                    .take(3)
                    .fold(String::new(), |a, b| a + b.trim() + " ")
                    + "...";

                info!("\t{}", preview);

                let _ = tx
                    .execute(statement.0.as_str(), &[])
                    .map_err(|e| e.to_string())?;
            }

            let _ = tx.execute(insert_revision.as_str(), &[
                &revision.created_at,
                &revision.checksum,
                &revision.filename,
                &revision.name,
            ]).map_err(|e| e.to_string())?;
        }

        if commit {
            tx.commit().map_err(|e| e.to_string())?;
        }

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
        info!("Creating schema {}", self.schema);
        let create = CREATE_SCHEMA.replace("$$schema$$", &self.schema);

        self.client.execute(create.as_str(), &[]).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn create_table(&mut self) -> Result<(), String> {
        info!("Creating table {}.{}", self.schema, self.table);
        let create = CREATE_TABLE
            .replace("$$schema$$", &self.schema)
            .replace("$$table$$", &self.table);

        self.client.execute(create.as_str(), &[]).map_err(|e| e.to_string())?;
        Ok(())
    }
}
