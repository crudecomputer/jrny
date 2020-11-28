use std::collections::HashMap;
use std::rc::Rc;

use crate::{AnnotatedRevision, RevisionRecord, RevisionFile};

#[derive(Debug)]
pub struct Review {
    annotated: Vec<AnnotatedRevision>,
    files: Vec<Rc<RevisionFile>>,
    records: Vec<Rc<RevisionRecord>>,
    files_map: HashMap<String, Rc<RevisionFile>>,
    records_map: HashMap<String, Rc<RevisionRecord>>,
}

impl Review {
    pub fn annotate(
        files: Vec<RevisionFile>,
        records: Vec<RevisionRecord>
    ) -> Vec<AnnotatedRevision> {
        let Self { mut annotated, .. } = Self::new(files, records).annotate_revisions();

        annotated.sort();
        annotated
    }

    fn new(mut files: Vec<RevisionFile>, mut records: Vec<RevisionRecord>) -> Self {
        let files: Vec<Rc<RevisionFile>> = files
            .drain(..)
            .map(|fr| Rc::new(fr))
            .collect();

        let files_map = files.iter()
            // TODO clean this stuff up
            .map(|rc_fr| (rc_fr.filename.clone(), rc_fr.clone()))
            .collect();

        let records: Vec<Rc<RevisionRecord>> = records
            .drain(..)
            .map(|dr| Rc::new(dr))
            .collect();

        let records_map = records.iter()
            .map(|rc_dr| (rc_dr.filename.clone(), rc_dr.clone()))
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
                name: file.name.clone(),
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
                name: record.name.clone(),
                on_disk: false,
            };

            self.annotated.push(anno);
        }

        self
    }
}
