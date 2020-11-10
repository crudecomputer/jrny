use std::path::PathBuf;

use crate::CONF;


pub struct ProjectPaths {
    pub conf: PathBuf,
    pub revisions: PathBuf,
    pub root: PathBuf,
}

impl ProjectPaths {
    pub fn for_new_project(root_dir: &str) -> Result<Self, String> {
        let root = PathBuf::from(root_dir);
        let revisions = root.join("revisions");
        let conf = root.join(CONF);

        let paths = ProjectPaths { conf, revisions, root };

        paths.valid_for_new()?;

        Ok(paths)
    }

    pub fn from_conf_path(conf_path_name: Option<&str>) -> Result<Self, String> {
        let conf = PathBuf::from(conf_path_name.unwrap_or(CONF));

        let root = conf.parent()
            .ok_or_else(|| "Config filepath is not valid".to_string())?
            .to_path_buf();

        let revisions = root.join("revisions");

        Ok(ProjectPaths { conf, revisions, root })
    }

    fn valid_for_new(&self) -> Result<(), String> {
        if self.root.exists() && !self.root.is_dir() {
            return Err(format!("{} is not a directory", self.root.display()));
        }

        if self.revisions.exists() && !Self::is_empty_dir(&self.revisions)? {
            return Err(format!("{} is not an empty directory", self.revisions.display()));
        }

        if self.conf.exists() {
            return Err(format!("{} already exists", self.conf.display()));
        }

        Ok(())
    }

           
    fn is_empty_dir(p: &PathBuf) -> Result<bool, String> {
        Ok(p.is_dir() && p.read_dir()
           .map_err(|e| e.to_string())?
           .next()
           .is_none())
    }
}
