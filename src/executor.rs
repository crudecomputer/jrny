use crate::ConnectionConfig;
use postgres::{Client, NoTls};
use std::time::Duration;

pub trait Executor: From<ConnectionConfig> {
    fn execute(&self);

    fn connect(&mut self);
}

pub struct PostgresExecutor {
    host: String,
    port: u16,
    name: String,
    user: String,
    pass: Option<String>,
    client: Option<Client>,
}

impl From<ConnectionConfig> for PostgresExecutor {
    fn from(conf: ConnectionConfig) -> Self {
        PostgresExecutor {
            host: conf.host,
            port: conf.port,
            name: conf.name,
            user: conf.user,
            pass: conf.pass,
            client: None,
        }
    }
}

impl Executor for PostgresExecutor {
    fn connect(&mut self) {
        let mut client = Client::configure();

        client.application_name("jrny")
            .connect_timeout(Duration::new(30, 0))
            .host(&self.host)
            .port(self.port)
            .dbname(&self.name)
            .user(&self.user);

        if let Some(password) = &self.pass {
            client.password(password);
        }

        self.client = Some(client
            .connect(NoTls)
            .expect("Could not connect to PostgreSQL"));
    }

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
