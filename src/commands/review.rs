use chrono::{DateTime, Utc};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::rc::Rc;

use crate::{DatabaseRevision, FileRevision};

#[derive(Debug, Eq)]
pub struct AnnotatedRevision {
    pub applied_on: Option<DateTime<Utc>>,
    pub checksum: Option<String>,
    pub contents: Option<String>,
    pub created_at: DateTime<Utc>,
    pub filename: String,
    pub checksums_match: Option<bool>,
    pub on_disk: bool,
}

impl Ord for AnnotatedRevision {
    fn cmp(&self, other: &Self) -> Ordering {
        (&self.created_at, &self.filename).cmp(&(&other.created_at, &other.filename))
    }
}

impl PartialOrd for AnnotatedRevision {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for AnnotatedRevision {
    fn eq(&self, other: &Self) -> bool {
        self.created_at == other.created_at && self.filename == other.filename
    }
}

#[derive(Debug)]
pub struct Review {
    annotated: Vec<AnnotatedRevision>,
    files: Vec<Rc<FileRevision>>,
    records: Vec<Rc<DatabaseRevision>>,
    files_map: HashMap<String, Rc<FileRevision>>,
    records_map: HashMap<String, Rc<DatabaseRevision>>,
}

impl Review {
    pub fn annotate(
        files: Vec<FileRevision>,
        records: Vec<DatabaseRevision>
    ) -> Vec<AnnotatedRevision> {
        let Self { mut annotated, .. } = Self::new(files, records).annotate_revisions();

        annotated.sort();
        annotated
    }

    fn new(mut files: Vec<FileRevision>, mut records: Vec<DatabaseRevision>) -> Self {
        let files: Vec<Rc<FileRevision>> = files
            .drain(..)
            .map(|fr| Rc::new(fr))
            .collect();

        let files_map = files.iter()
            // TODO clean this stuff up
            .map(|fr| (format!("{}.{}.sql", fr.created_at.timestamp(), fr.filename), fr.clone()))
            .collect();

        let records: Vec<Rc<DatabaseRevision>> = records
            .drain(..)
            .map(|dr| Rc::new(dr))
            .collect();

        let records_map = records.iter()
            .map(|dr| (format!("{}.{}.sql", dr.created_at.timestamp(), dr.filename), dr.clone()))
            .collect();

        Self { annotated: vec![], files, files_map, records, records_map }
    }

    /// Builds a list that represents all revisions files and records, matching each
    /// to determine which files have been applied and, for those that do, whether or
    /// not the checksums still match. Additionally, this verifies that all records
    /// continue to have corresponding files.
    pub fn annotate_revisions(mut self) -> Self {
        for file in self.files.iter() {
            let mut anno = AnnotatedRevision {
                applied_on: None,
                checksum: Some(file.checksum.clone()),
                checksums_match: None,
                contents: Some(file.contents.clone()),
                created_at: file.created_at.clone(),
                filename: file.filename.clone(),
                on_disk: true,
            };

            if let Some(record) = self.records_map.get(&file.filename) {
                anno.applied_on = Some(record.applied_on.clone());
                anno.checksums_match = Some(file.checksum == record.checksum);
            }

            self.annotated.push(anno);
        }

        for record in self.records.iter() {
            if let Some(_) = self.files_map.get(&record.filename) {
                continue;
            }

            let anno = AnnotatedRevision {
                applied_on: Some(record.applied_on),
                checksum: None,
                checksums_match: None,
                contents: None,
                created_at: record.created_at,
                filename: record.filename.clone(),
                on_disk: false,
            };

            self.annotated.push(anno);
        }

        self
    }
}
