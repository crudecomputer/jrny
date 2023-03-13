use std::collections::{HashMap, HashSet};
use std::fmt;
use std::path::Path;
use std::rc::Rc;

use chrono::{DateTime, Utc};

use crate::revisions::{RevisionFile, RevisionRecord};
use crate::{Executor, Result};


#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum RevisionProblem {
    DuplicateId,
    FileChanged,
    FileNotFound,
    PrecedesApplied,
}

impl fmt::Display for RevisionProblem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use RevisionProblem::*;

        write!(f, "{}", match self {
            DuplicateId => "Revision has a duplicate id",
            FileChanged => "File has changed after being applied",
            FileNotFound => "File could not be found",
            PrecedesApplied => "Later revisions have already been applied",
        })
    }
} 

/// Comprehensive metadata for a revision detected on disk or in the database.
#[derive(Debug)]
pub struct RevisionMeta {
    pub id: i32,
    /// Moment the revision was applied to the database
    pub applied_on: Option<DateTime<Utc>>,
    /// Contents of revision file, if present
    pub contents: Option<String>,
    /// Moment the revision was created
    pub created_at: DateTime<Utc>,
    /// The full name of the file, including id, timestamp, and extension
    pub filename: String,
    /// The name of the file, excluding id, timestamp, and extension
    pub name: String,
}

#[derive(Debug)]
pub struct RevisionCheck {
    pub meta: RevisionMeta,
    pub problems: HashSet<RevisionProblem>,
}

impl RevisionCheck {
    pub fn all(exec: &mut Executor, revision_dir: &Path) -> Result<Vec<Self>> {
        // FIXME: Clean this up
        let mut files = RevisionFile::all(revision_dir)?;
        let mut records = exec.load_revisions()?;

        // TODO: Rc shouldn't be necessary
        let files: Vec<Rc<RevisionFile>> = files.drain(..).map(Rc::new).collect();
        let files_map: HashMap<String, Rc<RevisionFile>> = files
            .iter()
            .map(|file_rc| (file_rc.filename.clone(), file_rc.clone()))
            .collect();

        let records: Vec<Rc<RevisionRecord>> = records.drain(..).map(Rc::new).collect();
        let records_map: HashMap<String, Rc<RevisionRecord>> = records
            .iter()
            .map(|record_rc| (record_rc.filename.clone(), record_rc.clone()))
            .collect();

        let mut checked = Vec::new();

        for file in files.iter() {
            let mut problems = HashSet::new();
            let mut meta = RevisionMeta {
                id: file.id,
                applied_on: None,
                contents: Some(file.contents.clone()),
                created_at: file.created_at,
                filename: file.filename.clone(),
                name: file.name.clone(),
            };

            if let Some(record) = records_map.get(&file.filename) {
                meta.applied_on = Some(record.applied_on);

                if file.checksum != record.checksum {
                    problems.insert(RevisionProblem::FileChanged);
                }
            }

            checked.push(RevisionCheck { meta, problems });
        }

        for record in records.iter() {
            if files_map.get(&record.filename).is_some() {
                continue;
            }

            let mut problems = HashSet::new();
            problems.insert(RevisionProblem::FileNotFound);

            let meta = RevisionMeta {
                id: record.id,
                applied_on: Some(record.applied_on),
                contents: None,
                created_at: record.created_at,
                filename: record.filename.clone(),
                name: record.name.clone(),
            };

            checked.push(RevisionCheck { meta, problems });
        }

        checked.sort_by_key(|rev| (rev.meta.id, rev.meta.created_at));

        let id_last_applied = checked
            .iter()
            .rev()
            .find(|rev| rev.meta.applied_on.is_some())
            .map(|rev| rev.meta.id);

        let mut previous_id = None;

        for rev in &mut checked {
            if let Some(last_applied) = id_last_applied {
                if rev.meta.id < last_applied && rev.meta.applied_on.is_none() {
                    rev.problems.insert(RevisionProblem::PrecedesApplied);
                }
            }
            match previous_id {
                Some(prev) => {
                    if prev == rev.meta.id {
                        rev.problems.insert(RevisionProblem::DuplicateId);
                    }
                },
                None => {
                    previous_id = Some(rev.meta.id);
                },
            }
        }

        Ok(checked)
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct RevisionSummary {
    pub duplicate_ids: usize,
    pub files_changed: usize,
    pub files_not_found: usize,
    pub preceding_applied: usize,
}

#[derive(Default)]
pub struct Review {
    pub revisions: Vec<RevisionCheck>,
    pub summary: RevisionSummary,
}

impl Review {
    pub fn failed(&self) -> bool {
        self.summary.duplicate_ids > 0 ||
        self.summary.files_changed > 0 ||
        self.summary.files_not_found > 0 ||
        self.summary.preceding_applied > 0
    }
}

pub fn check_revisions(exec: &mut Executor, revision_dir: &Path) -> Result<Review> {
    use RevisionProblem::*;

    exec.ensure_table_exists()?;

    let revisions = RevisionCheck::all(exec, revision_dir)?;
    let mut summary = RevisionSummary::default();

    for rev in &revisions {
        if rev.problems.contains(&DuplicateId) {
            summary.duplicate_ids += 1;
        }
        if rev.problems.contains(&FileChanged) {
            summary.files_changed += 1;
        }
        if rev.problems.contains(&FileNotFound) {
            summary.files_not_found += 1;
        }
        if rev.problems.contains(&PrecedesApplied) {
            summary.preceding_applied += 1;
        }
    }

    Ok(Review { revisions, summary })
}