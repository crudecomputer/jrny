use log::warn;
use std::convert::TryFrom;

use crate::{
    config::Config,
    executor::Executor,
    revisions::AnnotatedRevision,
    statements::StatementGroup,
};
use super::Review;

pub struct Embark {
    pub to_apply: Vec<AnnotatedRevision>,
}

impl Embark {
    pub fn prepare(config: &Config, exec: &mut Executor) -> Result<Self, String> {
        let Review { mut revisions, .. } = Review::annotated_revisions(config, exec)?;

        // If checksum comparison is missing, it hasn't been applied so ignore it
        let changed: Vec<_> = revisions.iter()
            .filter(|anno| !anno.checksums_match.unwrap_or(true))
            .collect();

        let missing: Vec<_> = revisions.iter()
            .filter(|anno| !anno.on_disk)
            .collect();

        if changed.len() > 0 || missing.len() > 0 {
            let mut msg = "Failed to run revisions".to_string();

            if changed.len() > 0 {
                msg.push_str(&format!("{} have changed since being applied", changed.len()));
            }

            if missing.len() > 0 {
                msg.push_str(&format!("{} are no longer present on disk", changed.len()));
            }

            return Err(msg);
        }

        let to_apply: Vec<_> = revisions
            .drain(..)
            .filter(|anno|
                anno.on_disk &&
                anno.applied_on.is_none()
            )
            .collect();

        Ok(Self { to_apply })
    }

    pub fn apply(self, exec: &mut Executor, commit: bool) -> Result<(), String> {
        // Parse all files into statements before printing or applying any
        let mut groups = vec![];

        for revision in self.to_apply {
            match StatementGroup::try_from(revision.contents.as_ref().unwrap().as_str()) {
                Ok(group) => {
                    groups.push((revision, group));
                },
                Err(e) => {
                    warn!("\nFound error in \"{}\"", revision.filename);
                    return Err(e);
                },
            }
        }

        let _ = exec.run_revisions(&groups, commit)?;

        Ok(())
    }
}
