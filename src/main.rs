use clap::clap_app;
use jrny;

fn main() {
    let mut app = clap_app! {jrny =>
        (about: "Data's a journey, so manage yours with jrny - simple, isolated PostgreSQL schema management")
        (version: "0.1.0")

        (@subcommand begin =>
            (about: "Sets up relevant files and directories for a new revision timeline")
            (@arg dirpath: +required "The directory in which to set up new project files - will be created if does not exist")
        )

        //(@subcommand revise =>
            //(about: "Generates a new versioned SQL revision from within project directory")
            //(@arg name: +required "Name of the revision step")
        //)

        //(@subcommand review =>
            //(about: "Determines the necessary revisions to apply from within project directory")
        //)

        //(@subcommand embark =>
            //(about: "Applies the necessary revisions from within project directory")
        //)
    };

    // TODO How to print helpp in absence of subcommand without cloning?
    let result = match app.clone().get_matches().subcommand() {
        ("begin", Some(subm)) => jrny::begin(subm.value_of("dirpath").unwrap()),
        ("revise", Some(subm)) => jrny::connect().revise(subm.value_of("name").unwrap()),
        //("review", Some(_)) => jrny::connect().review(),
        //("embark", Some(_)) => jrny::connect().embark(),
        _ => {
            app.print_help().expect("Failed to print help");
            Ok(())
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {:?}", e);
    }
}
