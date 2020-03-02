use clap::clap_app;
use jrny;


fn main() {
    let appm = clap_app!{jrny =>
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

    match appm.subcommand() {
        ("start", Some(subm)) => jrny::start(subm.value_of("dirpath").unwrap()),
        ("revise", Some(subm)) => jrny::connect().revise(subm.value_of("name").unwrap()),
        ("review", Some(_subm)) => jrny::connect().review(),
        ("embark", Some(_subm)) => jrny::connect().embark(),
        _ => unreachable!(),
    }
}
