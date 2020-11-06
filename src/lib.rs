//pub mod executor;

pub mod commands;
mod client;
mod config;
mod executor;
mod paths;

pub use paths::{PathWithName, ProjectPaths};
use config::Config;
use executor::Executor;

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
name = "plan"
"#
.as_bytes();
