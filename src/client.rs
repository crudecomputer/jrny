use postgres::NoTls;
use std::convert::TryFrom;
use std::str::FromStr;
use std::time::Duration;

pub use postgres::Client;
use postgres::config::Config as ClientConfig;

use crate::config::Config;

impl TryFrom<&Config> for Client {
    type Error = String;

    fn try_from(config: &Config) -> Result<Self, Self::Error> {
        let mut config = ClientConfig::from_str(config.settings.connection.database_url.as_str())
            .map_err(|e| e.to_string())?;

        config.application_name("jrny");

        if let None = config.get_connect_timeout() {
            config.connect_timeout(Duration::new(30, 0));
        }

        let client = config.connect(NoTls).map_err(|e| e.to_string())?;

        Ok(client)
    }
}
