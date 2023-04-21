mod commands;
pub mod context;
mod db;
mod error;
mod revisions;

pub use commands::{begin, embark, plan, review};
pub use context::Config;
pub use error::Error;

pub(crate) use db::executor::Executor;

/// The canonical result type used throughout the crate.
pub type Result<T> = std::result::Result<T, Error>;

/// (For CLI usage) The default name of the config file
pub const CONF: &str = "jrny.toml";

/// (For CLI usage) The default name of the environment file
pub const ENV: &str = "jrny-env.toml";

/// (For CLI usage) The default name of the example environment file
pub const ENV_EX: &str = "jrny-env.example.toml";
