//pub mod executor;

pub mod commands;
mod paths;

pub use paths::ProjectPaths;

//use crate::executor::{Executor, PostgresExecutor};
//use serde::Deserialize;

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

