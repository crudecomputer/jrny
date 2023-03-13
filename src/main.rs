use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{crate_version, Parser};
use log::{warn, Level, LevelFilter, Log, Metadata, Record};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use jrny::context::{Config, Environment};
use jrny::{Error as JrnyError, Result as JrnyResult, CONF, ENV};

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
    env: CliEnvironment,
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

impl TryFrom<CliConfig> for Config {
    type Error = JrnyError;

    fn try_from(cli_cfg: CliConfig) -> Result<Self, Self::Error> {
        let confpath = cli_cfg.filepath.unwrap_or_else(|| PathBuf::from(CONF));

        Self::from_filepath(&confpath)
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
    fn jrny_environment(self, cfg: &Config) -> JrnyResult<Environment> {
        let envpath = self
            .filepath
            .unwrap_or_else(|| cfg.revisions.directory.parent().unwrap().join(ENV));

        // This validates the env file, even if someone overrides it with the
        // database url flag. The file itself is optional as long as the
        // database url is supplied.
        let env_file = (match Environment::from_filepath(&envpath) {
            Ok(env) => Ok(Some(env)),
            Err(err) => match err {
                JrnyError::EnvNotFound => Ok(None),
                e => Err(e),
            },
        })?;

        match self.database_url {
            Some(url) => Ok(Environment::from_database_url(&url)),
            None => match env_file {
                Some(env) => Ok(env),
                None => Err(JrnyError::EnvNotFound),
            },
        }
    }
}

// Basic implementation of a Log, as none of the complexity of
// common crates is particularly necessary here and the CLI tool
// just wants to print info! and warn! as basic messages
//
// See: https://docs.rs/log/0.4.11/log/#implementing-a-logger
struct Logger;

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        if record.metadata().level() == Level::Warn {
            let mut stderr = StandardStream::stderr(ColorChoice::Always);

            stderr
                .set_color(ColorSpec::new().set_fg(Some(Color::Red)))
                .unwrap();
            write!(&mut stderr, "{}", record.args()).unwrap();

            stderr.set_color(ColorSpec::new().set_fg(None)).unwrap();
            writeln!(&mut stderr).unwrap();

            return;
        }

        let mut stdout = StandardStream::stdout(ColorChoice::Always);
        writeln!(&mut stdout, "{}", record.args()).unwrap();
    }

    fn flush(&self) {}
}

fn main() -> ExitCode {
    log::set_logger(&Logger)
        .map(|()| log::set_max_level(LevelFilter::Info))
        .map_err(|e| e.to_string())
        .unwrap();

    let opts: Opts = Opts::parse();

    let result = match opts.subcmd {
        SubCommand::Begin(cmd) => begin(cmd),
        SubCommand::Plan(cmd) => plan(cmd),
        SubCommand::Review(cmd) => review(cmd),
        SubCommand::Embark(cmd) => embark(cmd),
    };

    // Returning the result directly would debugs print the error and exit with an
    // appropriate code, but return an ExitCode instead so that there is
    // greater control over logging and formatting messages.
    //
    // See: https://doc.rust-lang.org/std/process/struct.ExitCode.html#impl-ExitCode
    match result {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            warn!("");
            warn!("{}", e);

            // TODO: More fine-grained error-dependent codes?
            // See: https://github.com/kevlarr/jrny/issues/33
            ExitCode::FAILURE
        }
    }
}

fn begin(cmd: Begin) -> JrnyResult<()> {
    jrny::begin(&cmd.dirpath)
}

fn plan(cmd: Plan) -> JrnyResult<()> {
    let cfg: Config = cmd.cfg.try_into()?;

    // TODO: Allow passing in file contents via command-line?
    jrny::plan(&cfg, &cmd.name, None)
}

fn review(cmd: Review) -> JrnyResult<()> {
    let cfg: Config = cmd.cfg.try_into()?;
    let env = cmd.env.jrny_environment(&cfg)?;

    jrny::review(&cfg, &env)
}

fn embark(cmd: Embark) -> JrnyResult<()> {
    let cfg: Config = cmd.cfg.try_into()?;
    let env = cmd.env.jrny_environment(&cfg)?;

    jrny::embark(&cfg, &env)
}
