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

        // If checksum comparison is missing, it hasn't been applied so ignore it
        let changed: Vec<_> = revisions
            .iter()
            .filter(|anno| !anno.checksums_match.unwrap_or(true))
            .collect();

        let missing: Vec<_> = revisions
            .iter()
            .filter(|anno| !anno.on_disk)
            .collect();

        if !changed.is_empty() || !missing.is_empty() {
            return Err(Error::RevisionsFailedReview {
                changed: changed.len(),
                missing: missing.len(),
            });
        }

        let to_apply: Vec<_> = revisions
            .drain(..)
            .filter(|anno| anno.on_disk && anno.applied_on.is_none())
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
