use std::path::PathBuf;

/// Stores a path with a displayable string to ensure that such a path
/// can always be printed without error.
pub struct PathWithName {
    pub path: PathBuf,
    pub name: String,
}

pub struct ProjectPaths {
    pub conf: PathWithName,
    pub revisions: PathWithName,
    pub root: PathWithName,
}

impl ProjectPaths {
    pub fn for_new_project(path: &str, conf_name: &str) -> Result<Self, String> {
        let root = PathBuf::from(path);
        let revisions = root.join("revisions");
        let conf = root.join(conf_name);

        let paths = ProjectPaths {
            conf: PathWithName {
                name: conf.to_str()
                    .expect("Could not generate name for config file")
                    .to_string(),
                path: conf,
            },
            revisions: PathWithName {
                name: revisions.to_str()
                    .expect("Could not generate name for revisions path")
                    .to_string(),
                path: revisions,
            },
            root: PathWithName {
                name: path.to_string(),
                path: root,
            },

        };

        paths.valid_for_new()?;

        Ok(paths)
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
