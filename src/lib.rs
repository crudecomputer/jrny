pub mod executor;

use crate::executor::{Executor, PostgresExecutor};
use serde::{Deserialize};
use std::{env, fs, path::{Path}};


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
    println!("Setting up project at {}", path);
    let dirpath = Path::new(path);

    if dirpath.exists() {
        if !dirpath.is_dir() {
            return Err(format!("{} is not a directory", path));
        }
    } else {
        // does dir need to explicitly be created?
        //match fs.create_dir
    }

    let confpath = dirpath.join(CONF);
    let confpath = confpath.as_path();


    if confpath.exists() {
        return Err(format!("{} already exists in given directory", CONF));
    }

    Ok(())
}
