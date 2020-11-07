//pub mod executor;

pub mod commands;
mod client;
mod config;
mod executor;
mod paths;
mod revision;

pub use paths::ProjectPaths;
use config::Config;
use executor::Executor;
use revision::FileRevision;

const CONF: &str = "jrny.toml";
const CONF_TEMPLATE: &[u8] =
r#"# jrny.toml

[connection]
host = "localhost"
port = 5432
name = "dbname"
user = "dbrole"

[table]
schema = "jrny"
name = "revision"
"#
.as_bytes();
