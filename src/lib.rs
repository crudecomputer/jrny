//pub mod executor;

pub mod commands;
mod paths;

pub use paths::ProjectPaths;

//use crate::executor::{Executor, PostgresExecutor};
//use serde::Deserialize;
use std::{
    fs,
    io::prelude::*,
    path::PathBuf,
    time::SystemTime,
    str::FromStr,
};

//#[derive(Clone, Debug, Deserialize)]
//pub struct Config {
    //pub app: AppConfig,
    //pub connection: ConnectionConfig,
//}

//#[derive(Clone, Debug, Deserialize)]
//pub struct AppConfig {
    //pub executor: String,
    //pub schema: String,
    //pub table: String,
//}

//#[derive(Clone, Debug, Deserialize)]
//pub struct ConnectionConfig {
    //pub host: String,
    //pub name: String,
    //pub port: u16,
    //pub user: String,
    //pub pass: Option<String>
//}

//pub struct Jrny<E: Executor> {
    //config: Config,
    //executor: E,
//}

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
//pub fn new() -> Jrny<impl Executor> {
    //let currdir = env::current_dir().expect("There was an error accessing the current directory");

    //let config: Config = {
        //let contents = fs::read_to_string(currdir.as_path().join(CONF))
            //.expect(&format!("Could not open {}", CONF));

        //toml::from_str(&contents).expect(&format!("Could not parse {}", CONF))
    //};

    //let executor = match config.app.executor.as_ref() {
        //"postgres" => PostgresExecutor::from(config.connection.clone()),
        //_ => panic!("Invalid executor"),
    //};

    //Jrny { config, executor }
//}

/// Accepts a name for the migration file and an optional path to a config file.
/// If no path is provided, it will add a timestamped SQL file relative to current
/// working directory; otherwise it will add file in a directory relative to config.
pub fn revise(name: &str, conf_path: Option<&str>) -> Result<(), String> {
    // Non-monotonic clock should be fine since precision isn't important.
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let mut revision_path = match conf_path {
        Some(cp) => {
            let mut conf_path = PathBuf::from_str(cp).map_err(|e| e.to_string())?;

            if !conf_path.pop() {
                return Err("Config filepath is not valid".to_string());
            }

            conf_path
        },
        None => PathBuf::new(),
    };

    let filename = format!("{}-{}.sql", timestamp, name);
    revision_path.push("revisions");
    revision_path.push(&filename);

    fs::File::create(&revision_path)
        .map_err(|e| e.to_string())?
        .write_all(format!("-- Journey revision\n--\n-- {}\n--\n\n", filename).as_bytes())
        .map_err(|e| e.to_string())?;

    println!("Created {}", revision_path.display());

    Ok(())
}
