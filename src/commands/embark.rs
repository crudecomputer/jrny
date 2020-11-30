use crate::{
    config::Config,
    executor::Executor,
    revisions::AnnotatedRevision,
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
}
