use log::info;

use super::Review;
use crate::context::Config;
use crate::revisions::AnnotatedRevision;
use crate::{Error, Executor, Result};

pub struct Embark {
    pub to_apply: Vec<AnnotatedRevision>,
}

impl Embark {
    pub fn prepare(cfg: &Config, exec: &mut Executor) -> Result<Self> {
        let Review { mut revisions, .. } =
            Review::annotated_revisions(exec, &cfg.revisions.directory)?;

        // TODO this is copypasta
        let mut last_applied_index = -1;

        for (i, revision) in revisions.iter().enumerate() {
            if revision.applied_on.is_some() {
                last_applied_index = i as isize;
            }
        }

        // This has gotten pretty inelegant, so refactor at some point,
        // and there's more copy-pasta with the "previous id" thing
        let (mut changed, mut duplicate_ids, mut missing, mut predate_applied) = (0, 0, 0, 0);
        let mut previous_id = None;

        for (i, revision) in revisions.iter().enumerate() {
            // If checksum comparison is missing, it hasn't been applied so ignore it
            if !revision.checksums_match.unwrap_or(true) {
                changed += 1;
            }

            if !revision.on_disk {
                missing += 1;
            }

            if revision.applied_on.is_none() && (i as isize) < last_applied_index {
                predate_applied += 1;
            }

            if previous_id == Some(revision.id) {
                duplicate_ids += 1;
            }

            previous_id = Some(revision.id);
        }

        if changed + duplicate_ids + missing + predate_applied > 0 {
            return Err(Error::RevisionsFailedReview {
                changed,
                duplicate_ids,
                missing,
                predate_applied,
            });
        }

        let to_apply: Vec<_> = revisions
            .drain(..)
            .filter(|anno| anno.applied_on.is_none())
            .collect();

        Ok(Self { to_apply })
    }

    pub fn apply(self, exec: &mut Executor) -> Result<()> {
        info!("Applying {} revision(s)\n", self.to_apply.len());

        for revision in &self.to_apply {
            info!("  {}", revision.filename);
            exec.run_revision(revision)?;
        }

        Ok(())
    }
}
