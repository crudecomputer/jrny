use postgres::NoTls;
use std::time::Duration;
use std::convert::TryFrom;

pub use postgres::Client;

use crate::Config;

impl TryFrom<&Config> for Client {
    type Error = String;

    fn try_from(conf: &Config) -> Result<Self, Self::Error> {
        let mut client = Self::configure();

        client.application_name("jrny")
            .connect_timeout(Duration::new(30, 0))
            .host(&conf.connection.host)
            .port(conf.connection.port)
            .dbname(&conf.connection.name)
            .user(&conf.connection.user);

        let client = client.connect(NoTls).map_err(|e| e.to_string())?;

        Ok(client)
    }
}
