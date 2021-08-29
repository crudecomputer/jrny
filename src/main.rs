use std::path::PathBuf;

use clap::{AppSettings, Clap, crate_version};
use log::{info, warn, LevelFilter};

use jrny::{commands, Logger, CONF, ENV};


/// PostgreSQL schema revisions made easy - just add SQL!
#[derive(Clap, Debug)]
#[clap(version = crate_version!())]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap, Debug)]
#[clap(setting = AppSettings::ColoredHelp)]
enum SubCommand {
    Begin(Begin),
    Plan(Plan),
    Review(Review),
    Embark(Embark),
}

/// Sets up relevant files and directories for a new revision timeline
#[derive(Clap, Debug)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Begin {
    /// The directory in which to set up new project files - will be created if does not exist
    dirpath: PathBuf,
}

/// Generates a timestamped SQL revision file
#[derive(Clap, Debug)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Plan {
    /// Path to TOML configuration file
    #[clap(short, long, default_value = CONF)]
    config: PathBuf,
}

/// Provides a summary of applied and pending revisions, including whether any applied have changed or are not found
#[derive(Clap, Debug)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Review {
    /// Path to TOML configuration file
    #[clap(short, long, default_value = CONF)]
    config: PathBuf,

    /// Database connection string
    #[clap(short, long)]
    database_url: Option<String>,

    /// Path to optional TOML environment file
    #[clap(short, long, default_value = ENV)]
    environment: PathBuf,
}

/// Applies pending revisions upon successful review
#[derive(Clap, Debug)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Embark {
    /// Path to TOML configuration file
    #[clap(short, long, default_value = CONF)]
    config: PathBuf,

    /// Database connection string
    #[clap(short, long)]
    database_url: Option<String>,

    /// Path to optional TOML environment file
    #[clap(short, long, default_value = ENV)]
    environment: PathBuf,
}

fn main() {
    log::set_logger(&Logger)
        .map(|()| log::set_max_level(LevelFilter::Info))
        .map_err(|e| e.to_string())
        .unwrap();
    
    let opts: Opts = Opts::parse();

    let result = match &opts.subcmd {
        SubCommand::Begin(cmd) => {
            commands::Begin::execute(&cmd.dirpath)
        },
        SubCommand::Plan(cmd) => {
            Ok(())
        },
        SubCommand::Review(cmd) => {
            Ok(())
        },
        SubCommand::Embark(cmd) => {
            Ok(())
        },
    };

    if let Err(e) = result {
        warn!("Error: {}", e);
    }

    /*
    // Gets a value for config if supplied by user, or defaults to "default.conf"
    println!("Value for config: {}", opts.config);
    println!("Using input file: {}", opts.input);

    // Vary the output based on how many times the user used the "verbose" flag
    // (i.e. 'myprog -v -v -v' or 'myprog -vvv' vs 'myprog -v'
    match opts.verbose {
        0 => println!("No verbose info"),
        1 => println!("Some verbose info"),
        2 => println!("Tons of verbose info"),
        _ => println!("Don't be ridiculous"),
    }

    // You can handle information about subcommands by requesting their matches by name
    // (as below), requesting just the name used, or both at the same time
    match opts.subcmd {
        SubCommand::Test(t) => {
            if t.debug {
                println!("Printing debug info...");
            } else {
                println!("Printing normally...");
            }
        }
    }
    */
}

/*
use clap::{clap_app, App};

fn main() {

    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Info))
        .map_err(|e| e.to_string())
        .unwrap();

    // This explicitly doesn't use `AppSettings::SubcommandRequired)` since that makes it
    // harder to print help by default in absence of a subcommand, rather than printing
    // an error that prompts to use `--help`
    let mut app = clap_app! {jrny =>
        (version: env!("CARGO_PKG_VERSION"))


        (@subcommand plan =>
            (@arg name: +required "Name of the revision")
            (@arg config: -c --config [FILE] +takes_value "Path to TOML config file")
        )

        (@subcommand review =>
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
*/
