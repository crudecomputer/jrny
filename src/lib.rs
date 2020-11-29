pub mod commands;
pub mod paths;
pub mod revisions;
pub mod statements;

mod client;
mod config;
mod executor;

pub use config::Config;
pub use executor::Executor;


const CONF: &str = "jrny.toml";
const CONF_TEMPLATE: &[u8] =
r#"# jrny.toml

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
