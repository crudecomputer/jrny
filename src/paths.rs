use std::path::PathBuf;

use crate::CONF;

/// Stores a path with a displayable string to ensure that such a path
/// can always be printed without error.
pub struct PathWithName {
    pub path: PathBuf,
    pub name: String,
}

impl PathWithName {
    pub fn new(debug_name: &str, path: PathBuf) -> Result<Self, String> {
        Ok(Self {
            name: path.to_str()
                .expect(format!("Could not generate name for {}", debug_name).as_str())
                .to_string(),
            path,
        })
    }
}

pub struct ProjectPaths {
    pub conf: PathWithName,
    pub revisions: PathWithName,
    pub root: PathWithName,
}

impl ProjectPaths {
    pub fn for_new_project(root_dir: &str) -> Result<Self, String> {
        let root = PathBuf::from(root_dir);
        let revisions = root.join("revisions");
        let conf = root.join(CONF);

        let paths = ProjectPaths::new(conf, revisions, root)?;

        paths.valid_for_new()?;

        Ok(paths)
    }

    pub fn from_conf_path(conf_path_name: Option<&str>) -> Result<Self, String> {
        let conf = PathBuf::from(conf_path_name.unwrap_or(CONF));

        let root = conf.parent()
            .ok_or_else(|| "Config filepath is not valid".to_string())?
            .to_path_buf();

        let revisions = root.join("revisions");

        Ok(ProjectPaths::new(conf, revisions, root)?)
    }

    fn new(conf: PathBuf, revisions: PathBuf, root: PathBuf) -> Result<Self, String> {
        Ok(Self {
            conf:      PathWithName::new("config file", conf)?,
            revisions: PathWithName::new("revisions directory", revisions)?,
            root:      PathWithName::new("target directory", root)?,
        })
    }

    fn valid_for_new(&self) -> Result<(), String> {
        if self.root.path.exists() && !self.root.path.is_dir() {
            return Err(format!("{} is not a directory", self.root.name));
        }

        if self.revisions.path.exists() && !Self::is_empty_dir(&self.revisions.path)? {
            return Err(format!("{} is not an empty directory", self.revisions.name));
        }

        if self.conf.path.exists() {
            return Err(format!("{} already exists", self.conf.name));
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
