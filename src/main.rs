use clap::clap_app;
use serde::{Serialize, Deserialize};
use std::{env, fs};
use toml;


const CONF: &str = "jrny.toml";


#[derive(Debug, Deserialize)]
struct Config {
    database: Database,
}

#[derive(Debug, Deserialize)]
struct Database {
    host: String,
    name: String,
    port: usize,

}

struct Jrny {
    config: Config,
}

impl Jrny {
    pub fn start(dirpath: &str) {
        println!("Setting up project at {}", dirpath);
    }

    /// Looks for config TOML file in current working directory and parses
    /// to set up new Jrny instance
    pub fn new() -> Self {
        let config: Config = {
            let currdir = env::current_dir()
                .expect("There was an error accessing the current directory");

            let contents = fs::read_to_string(currdir.as_path().join(CONF))
                .expect(&format!("Could not open {}", CONF));

            toml::from_str(&contents)
                .expect(&format!("Could not parse {}", CONF))
        };

        Jrny { config }
    }

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

fn main() {
    let jrnym = clap_app!{jrny =>
        (about: "Simple PostgreSQL schema management")
        (version: "0.1.0")

        (@subcommand start =>
            (about: "Sets up relevant files and directories for a new revision timeline")
            (@arg dirpath: +required "The directory in which to set up new project files - will be created if does not exist")
        )

        (@subcommand revise =>
            (about: "Generates a new versioned SQL revision from within project directory")
            (@arg name: +required "Name of the revision step")
        )

        (@subcommand review =>
            (about: "Determines the necessary revisions to apply from within project directory")
        )

        (@subcommand embark =>
            (about: "Applies the necessary revisions from within project directory")
        )
    }.get_matches();

    match jrnym.subcommand() {
        ("start", Some(subm)) => Jrny::start(subm.value_of("dirpath").unwrap()),
        ("revise", Some(subm)) => Jrny::new().revise(subm.value_of("name").unwrap()),
        ("review", Some(_subm)) => Jrny::new().review(),
        ("embark", Some(_subm)) => Jrny::new().embark(),
        _ => unreachable!(),
    }
}
