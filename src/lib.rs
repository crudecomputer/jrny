pub mod commands;
pub mod paths;
pub mod revisions;
pub mod statements;

mod client;
mod config;
mod error;
mod executor;
mod logger;
mod parser;

pub use config::Config;
pub use error::Error;
pub use executor::Executor;
pub use logger::Logger;

pub type Result<T> = std::result::Result<T, Error>;

const CONF: &str = "jrny.toml";
const CONF_TEMPLATE: &[u8] = r#"# jrny config

[connection]
# Specifies how `jrny` will connect to the target database. Available strategies are:
#
#   env-url-string:  Use a single connection string of any supported format, the value
#                    of which is read from an environment variable whose name is specified
#                    by `var-name`
#
# NOTE: Only this single strategy is supported currently, but this is likely to be
# extended in the future.
#
strategy = { type = "env-url-string", var-name = "JRNY_DATABASE_URL" }

[table]
# Specifies which schema and table `jrny` will use to track revision history.
schema = "public"
name = "jrny_revision"
"#
.as_bytes();
