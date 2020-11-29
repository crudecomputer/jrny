use std::collections::HashMap;
use std::rc::Rc;

use crate::CONF_TEMPLATE;
use crate::revisions::{AnnotatedRevision, RevisionRecord, RevisionFile};
use crate::paths::ProjectPaths;

use std::io::Write;
use std::fs;


pub struct Begin {
    pub paths: ProjectPaths,
    pub created_conf: bool,
    pub created_revisions: bool,
    pub created_root: bool,
}

impl Begin {
    /// Accepts a path string targeting a directory to set up project files:
    /// The directory will be created if it does not exist or will fail if
    /// pointing to an existing non-directory. This will then either verify
    /// that there is an empty `revisions` directory nested within it or
    /// create it if not already present. If any error occurs, any changes
    /// to the file system will be attempted to be reversed.
    pub fn new_project(p: &str) -> Result<Self, String> {
        Ok(Self {
            paths: ProjectPaths::for_new_project(p)?,
            created_conf: false,
            created_revisions: false,
            created_root: false,
        })
    }

    /// Attempts to create the project root directory if it doesn't exist,
    /// marking created as true if newly created.
    pub fn create_root(mut self) -> Result<Self, String> {
        if !self.paths.root.exists() {
            fs::create_dir(&self.paths.root).map_err(|e| e.to_string())?;
            self.created_root = true;
        }

        Ok(self)
    }

    /// Attempts to create the revisions directory if it doesn't exist,
    /// marking created as true if newly created.
    pub fn create_revisions(mut self) -> Result<Self, String> {
        if !self.paths.revisions.exists() {
            if let Err(e) = fs::create_dir(&self.paths.revisions) {
                self.revert()?;

                return Err(e.to_string());
            }

            self.created_revisions = true;
        }

        Ok(self)
    }

    /// Attempts to create the default configuration file. If a failure occurs,
    /// it will attempt to clean up any directory or file created during the command.
    pub fn create_conf(mut self) -> Result<Self, String> {
        let mut err = None;

        match fs::File::create(&self.paths.conf) {
            Ok(mut f) => {
                self.created_conf = true;

                if let Err(e) = f.write_all(CONF_TEMPLATE) {
                    err = Some(e.to_string());
                }
            }
            Err(e) => {
                err = Some(e.to_string());
            }
        }

        if let Some(e) = err {
            self.revert()?;

            return Err(e);
        }

        Ok(self)
    }

    /// Attempts to remove any directories or files created during command execution.
    fn revert(&self) -> Result<(), String> {
        if self.created_conf {
            fs::remove_file(&self.paths.conf).map_err(|e| e.to_string())?;
        }

        if self.created_revisions {
            fs::remove_dir(&self.paths.revisions).map_err(|e| e.to_string())?;
        }

        if self.created_root {
            fs::remove_dir(&self.paths.root).map_err(|e| e.to_string())?;
        }

        Ok(())
    }
}


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
