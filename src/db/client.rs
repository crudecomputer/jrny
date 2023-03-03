use std::{convert::TryFrom, str::FromStr, time::Duration};

pub use postgres::Client;
use postgres::{config::Config, NoTls};

use crate::Environment;

impl TryFrom<&Environment> for Client {
    type Error = crate::Error;

    fn try_from(env: &Environment) -> Result<Self, Self::Error> {
        let mut config = Config::from_str(&env.database.url)?;

        config.application_name("jrny");

        if config.get_connect_timeout().is_none() {
            config.connect_timeout(Duration::new(30, 0));
        }

        let client = config.connect(NoTls)?;

        Ok(client)
    }
}
