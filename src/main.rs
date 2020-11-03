use clap::clap_app;
use jrny::{self, commands};

fn main() {
    let mut app = clap_app! {jrny =>
        (about: "Data's a journey, so manage yours with jrny - simple, isolated PostgreSQL schema management")
        (version: "0.1.0")

        (@subcommand begin =>
            (about: "Sets up relevant files and directories for a new revision timeline")
            (@arg dirpath: +required "The directory in which to set up new project files - will be created if does not exist")
        )

        (@subcommand plan =>
            (about: "Generates a timestamped SQL revision plan")
            (@arg name: +required "Name of the revision")
            (@arg config: -c --config [FILE] +takes_value "Sets a custom config file")
        )

        //(@subcommand review =>
            //(about: "Determines the necessary revisions to apply from within project directory")
        //)

        //(@subcommand embark =>
            //(about: "Applies the necessary revisions from within project directory")
        //)
    };

    let result = match app.clone().get_matches().subcommand() {
        ("begin", Some(subm)) => commands::begin(subm.value_of("dirpath").unwrap()),
        ("plan", Some(subm)) => commands::plan(
            subm.value_of("name").unwrap(),
            subm.value_of("config"),
        ),
        //("review", Some(_)) => jrny::connect().review(),
        //("embark", Some(_)) => jrny::connect().embark(),

        // TODO How to print help in absence of subcommand without cloning?
        _ => {
            app.print_help().expect("Failed to print help");
            println!("");
            Ok(())
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {:?}", e);
    }
}
