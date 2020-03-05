pub mod executor;

use crate::executor::{Executor, PostgresExecutor};
use serde::Deserialize;
use std::{env, fs, io::prelude::*, path::{Path, PathBuf}, time::{Duration, SystemTime}};

const CONF: &str = "jrny.toml";
const CONF_TEMPLATE: &[u8] = r#"# jrny.toml

[app]
executor = "postgres"
schema = "public"
table = "jrny_revisions"

[connection]
host = "localhost"
port = 5432
name = "dbname"
user = "dbrole"
"#
.as_bytes();

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub app: AppConfig,
    pub connection: ConnectionConfig,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AppConfig {
    pub executor: String,
    pub schema: String,
    pub table: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ConnectionConfig {
    pub host: String,
    pub name: String,
    pub port: u16,
    pub user: String,
}

pub struct Jrny<E: Executor> {
    config: Config,
    executor: E,
}

impl<E: Executor> Jrny<E> {

    /// Creates a new timestamped revision file relative
    /// to current working directory.
    pub fn revise(&self, name: &str) -> Result<(), String> {
        // Non-monotonic clock should be fine since precision isn't important.
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::new(0, 0))
            .as_secs();

        let rev_path = format!("revisions/{}-{}.sql", timestamp, name);
        let rev_path = Path::new(&rev_path);

        fs::File::create(&rev_path)
            .map_err(|e| e.to_string())?
            .write_all(format!("-- {} \n-- bleh bleh", rev_path.display()).as_bytes())
            .map_err(|e| e.to_string())?;

        println!("Created {}", rev_path.display());

        Ok(())
    }

    pub fn review(&self) {
        // Summarize existing,  eg.  version in database, e tc.
        // Find new revisions
        println!("Reviewing available revisions");
    }

    pub fn embark(&self) {
        println!("Applying available revisions");
    }
}

/// Looks for config TOML file in current working directory and parses
/// to set up new Jrny instance, creating the appropriate database executor
pub fn connect() -> Jrny<impl Executor> {
    let currdir = env::current_dir().expect("There was an error accessing the current directory");

    let config: Config = {
        let contents = fs::read_to_string(currdir.as_path().join(CONF))
            .expect(&format!("Could not open {}", CONF));

        toml::from_str(&contents).expect(&format!("Could not parse {}", CONF))
    };

    let executor = match config.app.executor.as_ref() {
        "postgres" => PostgresExecutor::from(config.connection.clone()),
        _ => panic!("Invalid executor"),
    };

    Jrny { config, executor }
}

/// Accepts a path string targeting a directory to set up project files:
/// The directory will be created if it does not exist or will fail if
/// pointing to an existing non-directory. This will then either verify
/// that there is an empty `revisions` directory nested within it or
/// create it if not already present. If any error occurs, any changes
/// to the file system will be attempted to be reversed.
pub fn begin(path: &str) -> Result<(), String> {
    // For simplicity's sake, perform all checks prior to creating
    // any directories or files
    let root_path = Path::new(path);

    if root_path.exists() && !root_path.is_dir() {
        return Err(format!("{} is not a directory", path));
    }

    let revisions_path = root_path.join("revisions");
    let revisions_path = revisions_path.as_path();

    if revisions_path.exists() && !is_empty_dir(&revisions_path)? {
        return Err(format!(
            "{} is not an empty directory",
            revisions_path.to_str().unwrap()
        ));
    }

    let conf_path = root_path.join(CONF);
    let conf_path = conf_path.as_path();

    if conf_path.exists() {
        return Err(format!("{} already exists in given directory", CONF));
    }

    let mut created_root = false;
    let mut created_revisions = false;
    let mut created_conf = false;

    let clean = |cond, path| -> Result<(), String> {
        if cond {
            fs::remove_dir(path).map_err(|e| e.to_string())?;
        }

        Ok(())
    };

    if !root_path.exists() {
        fs::create_dir(&root_path).map_err(|e| e.to_string())?;
        created_root = true;
    }

    let clean_root = || clean(created_root, &root_path);

    if !revisions_path.exists() {
        if let Err(e) = fs::create_dir(&revisions_path) {
            clean_root()?;
            return Err(e.to_string());
        }

        created_revisions = true;
    }

    let clean_revisions = || clean(created_revisions, &revisions_path);

    let mut err = None;

    match fs::File::create(&conf_path) {
        Ok(mut f) => {
            created_conf = true;

            if let Err(e) = f.write_all(CONF_TEMPLATE) {
                err = Some(e.to_string());
            }
        }
        Err(e) => {
            err = Some(e.to_string());
        }
    }

    let clean_conf = || clean(created_conf, &conf_path);

    if let Some(e) = err {
        clean_conf()?;
        clean_revisions()?;
        clean_root()?;

        return Err(e);
    }

    println!("The journey has begun:");

    let print_file = |prefix: &str, created: bool, path: &Path| println!(
        "  {}{}{}",
        prefix,
        path.to_str().unwrap(),
        if created { " [created]" } else { "" },
    );

    print_file("", created_root, &root_path);
    print_file("├── ", created_revisions, &revisions_path);
    print_file("└── ", created_conf, &conf_path);
    println!("");


    Ok(())
}

fn is_empty_dir(p: &Path) -> Result<bool, String> {
    Ok(p.is_dir() && p.read_dir().map_err(|e| e.to_string())?.next().is_none())
}
