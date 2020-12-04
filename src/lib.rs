pub mod commands;
pub mod paths;
pub mod revisions;
pub mod statements;

mod client;
mod config;
mod executor;
mod logger;
mod parser;

pub use config::Config;
pub use executor::Executor;
pub use logger::Logger;

const CONF: &str = "jrny.toml";
const CONF_TEMPLATE: &[u8] = r#"# jrny config

[connection]
host = "localhost"
port = 5432
name = "dbname"
user = "dbrole"

[table]
schema = "public"
name = "jrny_revision"
"#
.as_bytes();
