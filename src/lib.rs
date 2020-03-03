pub mod executor;

use crate::executor::{Executor, PostgresExecutor};
use serde::{Deserialize};
use std::{env, fs, io::prelude::*, path::{Path}};


const CONF: &str = "jrny.toml";


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
    pub fn revise(&self, name: &str) {
        println!("Creating revision for {}", name);
    }

    pub fn review(&self, ) {
        // Summarize existing,  eg.  version in database, e tc.
        // Find new revisions
        println!("Reviewing available revisions");
    }

    pub fn embark(&self, ) {
        println!("Applying available revisions");
    }
}

/// Looks for config TOML file in current working directory and parses
/// to set up new Jrny instance, creating the appropriate database executor
pub fn connect() -> Jrny<impl Executor> {
    let currdir = env::current_dir()
        .expect("There was an error accessing the current directory");

    let config: Config = {
        let contents = fs::read_to_string(currdir.as_path().join(CONF))
            .expect(&format!("Could not open {}", CONF));

        toml::from_str(&contents)
            .expect(&format!("Could not parse {}", CONF))
    };

    let executor = match config.app.executor.as_ref() {
        "postgres" => PostgresExecutor::from(config.connection.clone()),
        _ => panic!("Invalid executor"),
    };

    Jrny { config, executor }
}

pub fn start(path: &str) -> Result<(), String> {
    let dir_path = Path::new(path);

    if !dir_path.exists() {
        fs::create_dir(&dir_path)
            .map_err(|e| e.to_string())?;
    } else if !dir_path.is_dir() {
        return Err(format!("{} is not a directory", path));
    }

    let rev_path = dir_path.join("revisions");
    let rev_path = rev_path.as_path();
    let mut created_revisions = false;

    if !rev_path.exists() {
        created_revisions = true;
        fs::create_dir(&rev_path)
            .map_err(|e| e.to_string())?;
    } else if !is_empty_dir(&rev_path) {
        return Err(format!("{} is not an empty directory", rev_path.to_str().unwrap()));
    }

    let conf_path = dir_path.join(CONF);
    let conf_path = conf_path.as_path();

    if conf_path.exists() {
        return Err(format!("{} already exists in given directory", CONF));
    }

    let mut created_conf = false;
    let mut err = None;

    match fs::File::create(&conf_path) {
        Ok(mut f) => {
            created_conf = true;

            if let Err(e) = f.write_all(conf_template().as_bytes()) {
                err = Some(e.to_string());
            }
        },
        Err(e) => {
            err = Some(e.to_string());
        },
    }

    if let Some(e) = err {
        if created_revisions {
            fs::remove_dir(&rev_path)
                .map_err(|e| e.to_string())?;
        }

        if created_conf {
            fs::remove_file(&conf_path)
                .map_err(|e| e.to_string())?;
        }

        return Err(e);
    }

    Ok(())
}

fn conf_template() -> &'static str {
r#"# jrny.toml

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
}

fn is_empty_dir(p: &Path) -> bool {
    p.is_dir() && p.read_dir().unwrap().next().is_none()
}
