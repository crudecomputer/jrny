use std::{collections::HashMap, path::Path, rc::Rc};

use crate::{
    executor::Executor,
    revisions::{AnnotatedRevision, RevisionFile, RevisionRecord},
    Result,
};

pub(super) struct Review {
    pub(super) revisions: Vec<AnnotatedRevision>,
    files: Vec<Rc<RevisionFile>>,
    records: Vec<Rc<RevisionRecord>>,
    files_map: HashMap<String, Rc<RevisionFile>>,
    records_map: HashMap<String, Rc<RevisionRecord>>,
}

impl Review {
    pub(super) fn annotated_revisions(exec: &mut Executor, revision_dir: &Path) -> Result<Self> {
        Ok(Self::new(exec, revision_dir)?.annotate())
    }

    fn new(exec: &mut Executor, revision_dir: &Path) -> Result<Self> {
        exec.ensure_table_exists()?;

        let mut files = RevisionFile::all_from_disk(revision_dir)?;
        let mut records = exec.load_revisions()?;

        let files: Vec<Rc<RevisionFile>> = files.drain(..).map(Rc::new).collect();

        let files_map = files
            .iter()
            .map(|file_rc| (file_rc.filename.clone(), file_rc.clone()))
            .collect();

        let records: Vec<Rc<RevisionRecord>> = records.drain(..).map(Rc::new).collect();

        let records_map = records
            .iter()
            .map(|record_rc| (record_rc.filename.clone(), record_rc.clone()))
            .collect();

        Ok(Self {
            revisions: vec![],
            files,
            files_map,
            records,
            records_map,
        })
    }

    /// Builds a list that represents all revisions files and records, matching each
    /// to determine which files have been applied and, for those that do, whether or
    /// not the checksums still match. Additionally, this verifies that all records
    /// continue to have corresponding files.
    fn annotate(mut self) -> Self {
        for file in self.files.iter() {
            let mut anno = AnnotatedRevision {
                id: file.id,
                applied_on: None,
                checksum: Some(file.checksum.clone()),
                checksums_match: None,
                contents: Some(file.contents.clone()),
                created_at: file.created_at,
                filename: file.filename.clone(),
                name: file.name.clone(),
                on_disk: true,
            };

            if let Some(record) = self.records_map.get(&file.filename) {
                anno.applied_on = Some(record.applied_on);
                anno.checksums_match = Some(file.checksum == record.checksum);
            }

            self.revisions.push(anno);
        }

        for record in self.records.iter() {
            if self.files_map.get(&record.filename).is_some() {
                continue;
            }

            let anno = AnnotatedRevision {
                id: record.id,
                applied_on: Some(record.applied_on),
                checksum: None,
                checksums_match: None,
                contents: None,
                created_at: record.created_at,
                filename: record.filename.clone(),
                name: record.name.clone(),
                on_disk: false,
            };

            self.revisions.push(anno);
        }

        self.revisions.sort();
        self
    }
}
