pub mod executor;

use crate::executor::{Executor, PostgresExecutor};
use serde::{Deserialize};
use std::{env, fs};


const CONF: &str = "jrny.toml";


#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub executor: String,
    pub database: DatabaseConfig,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DatabaseConfig {
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

    let executor = match config.executor.as_ref() {
        "postgres" => PostgresExecutor::from(config.database.clone()),
        _ => panic!("Invalid executor"),
    };

    Jrny { config, executor }
}

pub fn start(dirpath: &str) {
    println!("Setting up project at {}", dirpath);
}
