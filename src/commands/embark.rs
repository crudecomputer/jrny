use log::info;

use super::Review;
use crate::{
    config::Config, executor::Executor, revisions::AnnotatedRevision,
    Error, Result,
};

pub struct Embark {
    pub to_apply: Vec<AnnotatedRevision>,
}

impl Embark {
    pub fn prepare(config: &Config, exec: &mut Executor) -> Result<Self> {
        let Review { mut revisions, .. } = Review::annotated_revisions(config, exec)?;

        // TODO this is copypasta
        let mut last_applied_index = -1;

        for (i, revision) in revisions.iter().enumerate() {
            if revision.applied_on.is_some() {
                last_applied_index = i as isize;
            }
        }

        let (mut changed, mut missing, mut predate_applied) = (0, 0, 0);

        for (i, revision) in revisions.iter().enumerate() {
            if revision.applied_on.is_none() && (i as isize) < last_applied_index {
                predate_applied += 1;
            }

            // If checksum comparison is missing, it hasn't been applied so ignore it
            if !revision.checksums_match.unwrap_or(true) {
                changed += 1;
            }

            if !revision.on_disk {
                missing += 1;
            }
        }

        if changed + missing + predate_applied > 0 {
            return Err(Error::RevisionsFailedReview {
                changed,
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
            exec.run_revision(&revision)?;
        }

        Ok(())
    }
}
