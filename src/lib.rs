pub mod commands;

mod client;
mod config;
mod environment;
mod error;
mod executor;
mod logger;
mod revisions;

pub use commands::*;
pub use config::Config;
pub use environment::Environment;
pub use error::Error;
pub use logger::Logger;

// Crate result type
pub type Result<T> = std::result::Result<T, error::Error>;

/// The default name of the config file
pub const CONF: &str = "jrny.toml";

/// The default name of the environment file
pub const ENV: &str = "jrny-env.toml";

/// The default name of the example environment file
pub const ENV_EX: &str = "jrny-env.example.toml";
