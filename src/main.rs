use clap::{clap_app, App};
use log::{info, warn, LevelFilter};

use jrny::{self, Logger};

static LOGGER: Logger = Logger;

fn main() {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Info))
        .map_err(|e| e.to_string())
        .unwrap();

    // This explicitly doesn't use `AppSettings::SubcommandRequired)` since that makes it
    // harder to print help by default in absence of a subcommand, rather than printing
    // an error that prompts to use `--help`
    let mut app = clap_app! {jrny =>
        (about: "PostgreSQL schema revisions made easy - just add SQL")
        (version: env!("CARGO_PKG_VERSION"))

        (@subcommand begin =>
            (about: "Sets up relevant files and directories for a new revision timeline")
            (@arg dirpath: +required "The directory in which to set up new project files - will be created if does not exist")
        )

        (@subcommand plan =>
            (about: "Generates a timestamped SQL revision file")
            (@arg name: +required "Name of the revision")
            (@arg config: -c --config [FILE] +takes_value "Path to TOML config file")
        )

        (@subcommand review =>
            (about: "Provides a summary of applied and pending revisions, including whether any applied have changed or are not found")
            (@arg config: -c --config [FILE] +takes_value "Path to TOML config file")
        )

        (@subcommand embark =>
            (about: "Applies pending revisions upon successful review")
            (@arg config: -c --config [FILE] +takes_value "Path to TOML config file")
        )
    };

    let result = match app.clone().get_matches().subcommand() {
        ("begin", Some(cmd)) => jrny::begin(
            cmd.value_of("dirpath").unwrap()
        ),
        ("plan", Some(cmd)) => jrny::plan(
            cmd.value_of("name").unwrap(),
            cmd.value_of("config")
        ),
        ("review", Some(cmd)) => jrny::review(
            cmd.value_of("config")
        ),
        ("embark", Some(cmd)) => jrny::embark(
            cmd.value_of("config")
        ),
        ("", None) => {
            log_help(&mut app);
            Ok(())
        },
        _ => unreachable!(),
    };

    if let Err(e) = result {
        warn!("Error: {}", e);
    }
}

/// Uses Logger facade to print long help message, rather than
/// printing to stdout explicitly.
fn log_help(app: &mut App) {
    let msg = {
        let mut bytes = Vec::new();
        app.write_long_help(&mut bytes).unwrap();

        String::from_utf8(bytes).unwrap()
    };

    info!("{}", msg);
}
