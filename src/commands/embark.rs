use log::info;

use super::review::{CombinedRevision, Review};
use crate::context::Config;
use crate::{Error, Executor, Result};

pub struct Embark {
    // TODO: Don't need combined, only need on disk
    pub to_apply: Vec<CombinedRevision>,
}

impl Embark {
    pub fn prepare(cfg: &Config, exec: &mut Executor) -> Result<Self> {
        let Review { mut revisions, .. } =
            Review::new(exec, &cfg.revisions.directory)?;


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
