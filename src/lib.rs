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

[table]
schema = "public"
name = "jrny_plan"

[connection]
host = "localhost"
port = 5432
name = "dbname"
user = "dbrole"
"#
.as_bytes();
