use std::path::PathBuf;

use clap::{Clap, crate_version};
use log::{warn, LevelFilter};

use jrny::{
    commands,
    Config,
    Environment,
    Logger,
    Result as JrnyResult,
    CONF,
    ENV};


/// PostgreSQL schema revisions made easy - just add SQL!
#[derive(Clap, Debug)]
#[clap(version = crate_version!())]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap, Debug)]
enum SubCommand {
    Begin(Begin),
    Plan(Plan),
    Review(Review),
    Embark(Embark),
}

/// Sets up relevant files and directories for a new revision timeline
#[derive(Clap, Debug)]
struct Begin {
    /// The directory in which to set up new project files - will be created if does not exist
    dirpath: PathBuf,
}

/// Generates a new SQL revision file
#[derive(Clap, Debug)]
struct Plan {
    /// Name for the new revision file
    name: String,

    /// Path to TOML configuration file, defaulting to `jrny.toml`
    /// in current directory
    #[clap(short, long)]
    config: Option<PathBuf>,
}

/// Summarizes the state of revisions on disk and in database
#[derive(Clap, Debug)]
struct Review {
    /// Path to TOML configuration file, defaulting to `jrny.toml`
    /// in current directory
    #[clap(short, long)]
    config: Option<PathBuf>,

    /// Database connection string if not supplied by environment file
    /// or if overriding connection string from environment file
    #[clap(short, long)]
    database_url: Option<String>,

    /// Path to optional TOML environment file, defaulting to `jrny-env.toml`
    /// in directory relative to config file
    #[clap(short, long)]
    environment: Option<PathBuf>,
}

/// Applies pending revisions upon successful review
#[derive(Clap, Debug)]
struct Embark {
    /// Path to TOML configuration file, defaulting to `jrny.toml`
    /// in current directory
    #[clap(short, long)]
    config: Option<PathBuf>,

    /// Database connection string if not supplied by environment file
    /// or if overriding connection string from environment file
    #[clap(short, long)]
    database_url: Option<String>,

    /// Path to optional TOML environment file, defaulting to `jrny-env.toml`
    /// in directory relative to config file
    #[clap(short, long)]
    environment: Option<PathBuf>,
}

fn main() {
    log::set_logger(&Logger)
        .map(|()| log::set_max_level(LevelFilter::Info))
        .map_err(|e| e.to_string())
        .unwrap();
    
    let opts: Opts = Opts::parse();

    let result = match opts.subcmd {
        SubCommand::Begin(cmd) => {
            commands::begin(&cmd.dirpath)
        },
        SubCommand::Plan(cmd) => {
            plan(cmd)
        },
        SubCommand::Review(cmd) => {
            review(cmd)
        },
        SubCommand::Embark(cmd) => {
            embark(cmd)
        },
    };

    if let Err(e) = result {
        warn!("Error: {}", e);
    }
}

fn plan(cmd: Plan) -> JrnyResult<()> {
    let confpath = cmd.config.unwrap_or_else(|| PathBuf::from(CONF));
    let cfg = Config::from_filepath(&confpath)?;

    commands::plan(&cfg, &cmd.name)
}

fn review(cmd: Review) -> JrnyResult<()> {
    let confpath = cmd.config.unwrap_or_else(|| PathBuf::from(CONF));

    let cfg = Config::from_filepath(&confpath)?;

    let envpath = cmd.environment
        .as_ref()
        .cloned()
        .unwrap_or_else(|| cfg.revisions.directory
            .parent()
            .unwrap()
            .join(ENV)
        );

    let mut env = Environment::from_filepath(&envpath)?;

    if let Some(url) = cmd.database_url {
        env.database.url = url.clone();
    }

    commands::review(&cfg, &env)
}

fn embark(cmd: Embark) -> JrnyResult<()> {
    let confpath = cmd.config.unwrap_or_else(|| PathBuf::from(CONF));

    let cfg = Config::from_filepath(&confpath)?;

    let envpath = cmd.environment
        .as_ref()
        .cloned()
        .unwrap_or_else(|| cfg.revisions.directory
            .parent()
            .unwrap()
            .join(ENV)
        );

    let mut env = Environment::from_filepath(&envpath)?;

    if let Some(url) = cmd.database_url {
        env.database.url = url.clone();
    }

    commands::embark(&cfg, &env)
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
