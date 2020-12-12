use log::warn;
use std::convert::TryFrom;

use super::Review;
use crate::{
    Error,
    Result,
    config::Config,
    executor::Executor,
    revisions::AnnotatedRevision,
    statements::StatementGroup,
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

    /// Parse all pending revisions into individual statements and then applies each.
    pub fn apply(self, exec: &mut Executor, commit: bool) -> Result<()> {
        let mut groups = vec![];

        for revision in self.to_apply {
            // If `unwrap` panics then it's actually a bug, since all revisions at
            // this point SHOULD be loaded from disk and hence SHOULD have content.
            let contents = revision.contents.as_ref().unwrap().as_str();

            match StatementGroup::try_from(contents) {
                Ok(group) => {
                    groups.push((revision, group));
                }
                Err(e) => {
                    warn!("\nFound error in \"{}\"", revision.filename);
                    return Err(e);
                }
            }
        }

        exec.run_revisions(&groups, commit)?;

        Ok(())
    }
}
