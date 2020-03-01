use clap::clap_app;


mod jrny {
    pub fn new(dirpath: &str) {
        println!("Setting up project at {}", dirpath);
    }

    pub fn revise(name: &str) {
        println!("Creating revision for {}", name);
    }

    pub fn review() {
        println!("Reviewing available revisions");
    }

    pub fn embark() {
        println!("Applying available revisions");
    }
}

fn main() {
    let appm = clap_app!{jrny =>
        (version: "0.1.0")
        (about: "Simple PostgreSQL schema management")
        (@subcommand new =>
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

    match appm.subcommand() {
        ("new", Some(subm)) => jrny::new(subm.value_of("dirpath").unwrap()),
        ("revise", Some(subm)) => jrny::revise(subm.value_of("name").unwrap()),
        ("review", Some(_subm)) => jrny::review(),
        ("embark", Some(_subm)) => jrny::embark(),
        _ => unreachable!(),
    }
}
