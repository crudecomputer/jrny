//pub mod executor;

pub mod commands;
mod paths;

pub use paths::{PathWithName, ProjectPaths};

//use crate::executor::{Executor, PostgresExecutor};
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;


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
    //pub pass: Option<String>
}

//pub struct Jrny<E: Executor> {
pub struct Jrny {
    config: Config,
    //executor: E,
}

impl Jrny {
    pub fn from_config(conf_path_name: Option<&str>) -> Result<Self, String> {
        let conf = PathWithName::new("config file", PathBuf::from(
            if let Some(name) = conf_path_name { name } else { CONF }
        ))?;

        let config: Config = {
            let contents = fs::read_to_string(conf.path)
                .expect(&format!("Could not open {}", conf.name));

            toml::from_str(&contents).expect(&format!("Could not parse {}", CONF))
        };

        Ok(Self { config })
    }
}

//impl<E: Executor> Jrny<E> {

    //pub fn review(&self) {
        //// Summarize existing,  eg.  version in database, e tc.
        //// Find new revisions
        //println!("Reviewing available revisions");
    //}

    //pub fn embark(&self) {
        //println!("Applying available revisions");
    //}
//}

// Looks for config TOML file in current working directory and parses
// to set up new Jrny instance, creating the appropriate database executor
//pub fn new() -> Jrny {
    //let currdir = env::current_dir().expect("There was an error accessing the current directory");


    //let executor = match config.app.executor.as_ref() {
        //"postgres" => PostgresExecutor::from(config.connection.clone()),
        //_ => panic!("Invalid executor"),
    //};

    //Jrny { config, executor }
//}

