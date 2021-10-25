use std::path::PathBuf;

use clap::{Clap, crate_version};
use log::{warn, LevelFilter};

use jrny::{
    CONF,
    ENV,
    Config,
    Environment,
    Logger,
    Result as JrnyResult,
};


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
        SubCommand::Begin(cmd)  => begin(cmd),
        SubCommand::Plan(cmd)   => plan(cmd),
        SubCommand::Review(cmd) => review(cmd),
        SubCommand::Embark(cmd) => embark(cmd),
    };

    if let Err(e) = result {
        warn!("Error: {}", e);
    }
}

fn begin(cmd: Begin) -> JrnyResult<()> {
    jrny::begin(&cmd.dirpath)
}

fn plan(cmd: Plan) -> JrnyResult<()> {
    let confpath = cmd.config.unwrap_or_else(|| PathBuf::from(CONF));
    let cfg = Config::from_filepath(&confpath)?;

    jrny::plan(&cfg, &cmd.name)
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

    jrny::review(&cfg, &env)
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

    jrny::embark(&cfg, &env)
}
