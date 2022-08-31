use std::path::PathBuf;

use clap::{Parser, crate_version};
use log::{warn, LevelFilter};

use jrny::{
    CONF,
    ENV,
    Config,
    Environment,
    Error as JrnyError,
    Logger,
    Result as JrnyResult,
};

/// PostgreSQL schema revisions made easy - just add SQL!
#[derive(Parser, Debug)]
#[clap(version = crate_version!())]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser, Debug)]
enum SubCommand {
    Begin(Begin),
    Plan(Plan),
    Review(Review),
    Embark(Embark),
}

/// Sets up relevant files and directories for a new revision timeline
#[derive(Parser, Debug)]
struct Begin {
    /// The directory in which to set up new project files - will be created if does not exist
    dirpath: PathBuf,
}

/// Generates a new SQL revision file
#[derive(Parser, Debug)]
struct Plan {
    #[clap(flatten)]
    cfg: CliConfig,

    /// Name for the new revision file
    name: String,
}

/// Summarizes the state of revisions on disk and in database
#[derive(Parser, Debug)]
struct Review {
    #[clap(flatten)]
    cfg: CliConfig,

    #[clap(flatten)]
    env: CliEnvironment
}

/// Applies pending revisions upon successful review
#[derive(Parser, Debug)]
struct Embark {
    #[clap(flatten)]
    cfg: CliConfig,

    #[clap(flatten)]
    env: CliEnvironment,
}

#[derive(Parser, Debug)]
struct CliConfig {
    /// Path to TOML configuration file, defaulting to `jrny.toml`
    /// in current directory
    #[clap(short = 'c', long = "config", name = "CFG")]
    filepath: Option<PathBuf>,
}

impl CliConfig {
    fn to_cfg(self) -> JrnyResult<Config> {
        let confpath = self.filepath.unwrap_or_else(|| PathBuf::from(CONF));
        Config::from_filepath(&confpath)
    }
}

#[derive(Parser, Debug)]
struct CliEnvironment {
    /// Database connection string if overriding value from or not using
    /// an environment file.
    #[clap(short = 'd', long = "database-url", name = "URL")]
    database_url: Option<String>,

    /// Path to optional TOML environment file, defaulting to `jrny-env.toml`
    /// in directory relative to config file
    #[clap(short = 'e', long = "environment", name = "ENV")]
    filepath: Option<PathBuf>,
}

// Can't implement from/into traits if `Config` is involved, since it's technically foreign
impl CliEnvironment {
    fn to_env(self, cfg: &Config) -> JrnyResult<Environment> {
        let envpath = self.filepath
            .unwrap_or_else(|| cfg.revisions.directory
                .parent()
                .unwrap()
                .join(ENV)
            );

        // This validates the env file, even if someone overrides it with the
        // database url flag. The file itself is optional as long as the
        // database url is supplied.
        let env_file = (match Environment::from_filepath(&envpath) {
            Ok(env) => Ok(Some(env)),
            Err(err) => match err {
                JrnyError::EnvNotFound => Ok(None),
                e => Err(e),
            }
        })?;

        match self.database_url {
            Some(url) => Ok(Environment::from_database_url(&url)),
            None => match env_file {
                Some(env) => Ok(env),
                None => Err(JrnyError::EnvNotFound),
            }
        }
    }
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
        std::process::exit(101);
    }
}

fn begin(cmd: Begin) -> JrnyResult<()> {
    jrny::begin(&cmd.dirpath)
}

fn plan(cmd: Plan) -> JrnyResult<()> {
    let cfg = cmd.cfg.to_cfg()?;

    jrny::plan(&cfg, &cmd.name)
}

fn review(cmd: Review) -> JrnyResult<()> {
    let cfg = cmd.cfg.to_cfg()?;
    let env = cmd.env.to_env(&cfg)?;

    jrny::review(&cfg, &env)
}

fn embark(cmd: Embark) -> JrnyResult<()> {
    let cfg = cmd.cfg.to_cfg()?;
    let env = cmd.env.to_env(&cfg)?;

    jrny::embark(&cfg, &env)
}
