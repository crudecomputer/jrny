use std::path::PathBuf;

use clap::{AppSettings, Clap};

/// Generates a timestamped SQL revision file
#[derive(Clap, Debug)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Plan {
    /// Path to TOML configuration file
    #[clap(short, long, default_value = "jrny.toml")]
    config: PathBuf,
}
