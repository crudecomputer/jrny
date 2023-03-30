use std::collections::{HashMap, HashSet};
use std::fmt;
use std::path::Path;

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

        write!(
            f,
            "{}",
            match self {
                DuplicateId => "Revision has a duplicate id",
                FileChanged => "File has changed after being applied",
                FileNotFound => "File could not be found",
                PrecedesApplied => "Later revisions have already been applied",
            }
        )
    }
}

type RevisionProblems = HashSet<RevisionProblem>;

#[derive(Debug)]
enum ReviewItemSource {
    FileAndRecord {
        file: RevisionFile,
        record: RevisionRecord,
    },
    FileOnly(RevisionFile),
    RecordOnly(RevisionRecord),
}

use ReviewItemSource::*;

#[derive(Debug)]
pub struct ReviewItem {
    // pub meta: RevisionMeta,
    source: ReviewItemSource,

    /// Problems identified with the revision
    problems: HashSet<RevisionProblem>,
}

impl ReviewItem {
    pub fn id(&self) -> i32 {
        match &self.source {
            FileAndRecord { file, .. } | FileOnly(file) => file.id,
            RecordOnly(record) => record.id,
        }
    }

    pub fn name(&self) -> &str {
        match &self.source {
            FileAndRecord { file, .. } | FileOnly(file) => &file.name,
            RecordOnly(record) => &record.name,
        }
    }

    pub fn created_at(&self) -> &DateTime<Utc> {
        match &self.source {
            FileAndRecord { file, .. } | FileOnly(file) => &file.created_at,
            RecordOnly(record) => &record.created_at,
        }
    }

    pub fn problems(&self) -> &RevisionProblems {
        &self.problems
    }

    pub fn applied_on(&self) -> Option<&DateTime<Utc>> {
        match &self.source {
            FileAndRecord { record, .. } | RecordOnly(record) => Some(&record.applied_on),
            _ => None,
        }
    }

    pub fn applied(&self) -> bool {
        !self.pending()
    }

    pub fn pending(&self) -> bool {
        // Wow, clippy.. impressive
        // if let FileOnly(_) = self.source { true } else { false }
        matches!(self.source, FileOnly(_))
    }

    fn file_and_record(
        file: RevisionFile,
        record: RevisionRecord,
        problems: RevisionProblems,
    ) -> Self {
        Self {
            source: ReviewItemSource::FileAndRecord { file, record },
            problems,
        }
    }

    fn file_only(file: RevisionFile, problems: RevisionProblems) -> Self {
        Self {
            source: ReviewItemSource::FileOnly(file),
            problems,
        }
    }

    fn record_only(record: RevisionRecord, problems: RevisionProblems) -> Self {
        Self {
            source: ReviewItemSource::RecordOnly(record),
            problems,
        }
    }

    fn from_sources(files: Vec<RevisionFile>, records: Vec<RevisionRecord>) -> Vec<Self> {
        let mut items = Vec::new();

        // For extracting the equivalent record when iterating through files
        let mut records: HashMap<String, RevisionRecord> = records
            .into_iter()
            .map(|record| (record.name.clone(), record))
            .collect();

        for file in files {
            let mut problems = HashSet::new();
            let item = match records.remove(&file.name) {
                Some(record) => {
                    if file.checksum != record.checksum {
                        problems.insert(RevisionProblem::FileChanged);
                    }
                    Self::file_and_record(file, record, problems)
                }
                None => Self::file_only(file, problems),
            };

            items.push(item);
        }

        for record in records.into_values() {
            // Any records still present will not have a corresponding file
            let mut problems = HashSet::new();
            problems.insert(RevisionProblem::FileNotFound);
            items.push(Self::record_only(record, problems));
        }

        // Sort the items now to look for other problems, like duplicate ids
        // or pending revisions existing in sequence before applied ones
        items.sort_by_key(|item| (item.id(), item.created_at().to_owned()));

        let id_last_applied = items
            .iter()
            .rev()
            .find(|item| item.applied())
            .map(|item| item.id());

        let mut previous: Option<&mut ReviewItem> = None;

        for item in &mut items {
            if let Some(last_applied) = id_last_applied {
                // FIXME: What about when there are duplicate ids and only one is applied?
                if item.id() < last_applied && item.pending() {
                    item.problems.insert(RevisionProblem::PrecedesApplied);
                }
            }
            match &mut previous {
                Some(prev) => {
                    if item.id() == prev.id() {
                        prev.problems.insert(RevisionProblem::DuplicateId);
                        item.problems.insert(RevisionProblem::DuplicateId);
                    }
                }
                None => {
                    previous = Some(item);
                }
            }
        }

        items
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct ReviewSummary {
    duplicate_ids: usize,
    files_changed: usize,
    files_not_found: usize,
    preceding_applied: usize,
}

impl ReviewSummary {
    pub fn duplicate_ids(&self) -> usize {
        self.duplicate_ids
    }

    pub fn files_changed(&self) -> usize {
        self.files_changed
    }

    pub fn files_not_found(&self) -> usize {
        self.files_not_found
    }

    pub fn preceding_applied(&self) -> usize {
        self.preceding_applied
    }
}

#[derive(Default)]
pub struct Review {
    items: Vec<ReviewItem>,
    summary: ReviewSummary,
}

impl Review {
    pub fn failed(&self) -> bool {
        self.summary.duplicate_ids > 0
            || self.summary.files_changed > 0
            || self.summary.files_not_found > 0
            || self.summary.preceding_applied > 0
    }

    pub fn items(&self) -> &Vec<ReviewItem> {
        &self.items
    }

    pub fn summary(&self) -> &ReviewSummary {
        &self.summary
    }

    pub fn pending_revisions(&self) -> Vec<&RevisionFile> {
        self.items
            .iter()
            .filter_map(|item| match &item.source {
                FileOnly(file) => Some(file),
                _ => None,
            })
            .collect()
    }

    pub fn new(exec: &mut Executor, revision_dir: &Path) -> Result<Self> {
        use RevisionProblem::*;

        exec.ensure_table_exists()?;

        let files = RevisionFile::all(revision_dir)?;
        let records = exec.load_revisions()?;

        let items = ReviewItem::from_sources(files, records);
        let mut summary = ReviewSummary::default();

        for item in &items {
            if item.problems.contains(&DuplicateId) {
                summary.duplicate_ids += 1;
            }
            if item.problems.contains(&FileChanged) {
                summary.files_changed += 1;
            }
            if item.problems.contains(&FileNotFound) {
                summary.files_not_found += 1;
            }
            if item.problems.contains(&PrecedesApplied) {
                summary.preceding_applied += 1;
            }
        }

        Ok(Review { items, summary })
    }
}
