/*
use postgres::NoTls;
use std::convert::TryFrom;
use std::str::FromStr;
use std::time::Duration;

use postgres::config::Config as ClientConfig;
pub use postgres::Client;

use crate::config::Config;

impl TryFrom<&Config> for Client {
    type Error = crate::Error;

    fn try_from(config: &Config) -> Result<Self, Self::Error> {
        let mut config = ClientConfig::from_str(config.settings.connection.database_url.as_str())?;

        config.application_name("jrny");

        if config.get_connect_timeout().is_none() {
            config.connect_timeout(Duration::new(30, 0));
        }

        let client = config.connect(NoTls)?;

        Ok(client)
    }
}
*/
