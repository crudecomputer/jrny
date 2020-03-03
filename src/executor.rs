use crate::ConnectionConfig;
use postgres::{Client, NoTls};
use std::time::Duration;

pub trait Executor: From<ConnectionConfig> {
    fn execute(&self);
}

pub struct PostgresExecutor {
    client: Client,
}

impl From<ConnectionConfig> for PostgresExecutor {
    fn from(conf: ConnectionConfig) -> Self {
        let client = Client::configure()
            .application_name("jrny")
            .connect_timeout(Duration::new(30, 0))
            .host(&conf.host)
            .port(conf.port)
            .dbname(&conf.name)
            .user(&conf.user)
            .connect(NoTls)
            .expect("Could not connect to PostgreSQL");

        PostgresExecutor { client }
    }
}

impl Executor for PostgresExecutor {
    fn execute(&self) {
        /*
        client.batch_execute("
            CREATE TABLE person (
                id      SERIAL PRIMARY KEY,
                name    TEXT NOT NULL,
                data    BYTEA
            )
        ")?;


        let name = "Ferris";
        let data = None::<&[u8]>;
        client.execute(
            "INSERT INTO person (name, data) VALUES ($1, $2)",
            &[&name, &data],
        )?;

        for row in client.query("SELECT id, name, data FROM person", &[])? {
            let id: i32 = row.get(0);
            let name: &str = row.get(1);
            let data: Option<&[u8]> = row.get(2);

            println!("found person: {} {} {:?}", id, name, data);
        }
        */
    }
}
