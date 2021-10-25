pub mod commands;

mod client;
mod config;
mod environment;
mod error;
mod executor;
mod logger;
mod revisions;

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

/*

pub fn embark(conf_path_name: Option<&str>) -> Result<()> {
    let config = Config::new(conf_path_name)?;
    let mut exec = Executor::new(&config)?;

    let cmd = commands::Embark::prepare(&config, &mut exec)?;

    if cmd.to_apply.is_empty() {
        info!("No revisions to apply");
        return Ok(());
    }

    cmd.apply(&mut exec)?;

    Ok(())
}

/// Prints path string with optional prefix and "[created]" suffix if the created
/// condition is true.
fn print_path(prefix: &str, path: &PathBuf, created: bool) {
    info!(
        "{}{}{}",
        prefix,
        path.display(),
        if created { " [created]" } else { "" },
    );
}
*/
