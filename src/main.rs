use std::path::PathBuf;

use clap::{AppSettings, Clap, crate_version};
use log::{info, warn, LevelFilter};

use jrny::{commands, Logger, CONF, ENV};


/// PostgreSQL schema revisions made easy - just add SQL!
#[derive(Clap, Debug)]
#[clap(version = crate_version!())]
//#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap, Debug)]
//#[clap(setting = AppSettings::ColoredHelp)]
enum SubCommand {
    Begin(Begin),
    Plan(Plan),
    Review(Review),
    Embark(Embark),
}

/// Sets up relevant files and directories for a new revision timeline
#[derive(Clap, Debug)]
//#[clap(setting = AppSettings::ColoredHelp)]
struct Begin {
    /// The directory in which to set up new project files - will be created if does not exist
    dirpath: PathBuf,
}

/// Generates a new SQL revision file
#[derive(Clap, Debug)]
//#[clap(setting = AppSettings::ColoredHelp)]
struct Plan {
    /// Name for the new revision file
    name: String,

    /// Path to TOML configuration file
    #[clap(short, long, default_value = CONF)]
    config: PathBuf,
}

/// Summarizes the state of revisions on disk and in database
#[derive(Clap, Debug)]
//#[clap(setting = AppSettings::ColoredHelp)]
struct Review {
    #[clap(flatten)]
    context: Context,
}

/// Applies pending revisions upon successful review
#[derive(Clap, Debug)]
//#[clap(setting = AppSettings::ColoredHelp)]
struct Embark {
    #[clap(flatten)]
    context: Context,
}

#[derive(clap::Args, Debug)]
struct Context {
    /// Path to TOML configuration file
    #[clap(short, long, default_value = CONF)]
    config: PathBuf,

    /// Database connection string if not supplied by environment file
    /// or if overriding connection string from environment file
    #[clap(short, long)]
    database_url: Option<String>,

    /// Path to optional TOML environment file
    #[clap(short, long)]
    environment: Option<PathBuf>,
}

fn main() {
    log::set_logger(&Logger)
        .map(|()| log::set_max_level(LevelFilter::Info))
        .map_err(|e| e.to_string())
        .unwrap();
    
    let opts: Opts = Opts::parse();

    let result = match &opts.subcmd {
        SubCommand::Begin(cmd) => {
            commands::begin(&cmd.dirpath)
        },
        SubCommand::Plan(cmd) => {
            commands::plan(&cmd.config, &cmd.name)
        },
        SubCommand::Review(cmd) => {
            //commands::review(commands::ReviewArgs {
                //confpath: cmd.context.config.clone(),
                //envpath: cmd.context.environment.clone(),
                //database_url: cmd.context.database_url.clone(),
            //})
            Ok(())
        },
        SubCommand::Embark(cmd) => {
            Ok(())
        },
    };

    if let Err(e) = result {
        warn!("Error: {}", e);
    }
}

/*
use clap::{clap_app, App};

fn main() {

    // This explicitly doesn't use `AppSettings::SubcommandRequired)` since that makes it
    // harder to print help by default in absence of a subcommand, rather than printing
    // an error that prompts to use `--help`
    let mut app = clap_app! {jrny =>
        (version: env!("CARGO_PKG_VERSION"))


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
