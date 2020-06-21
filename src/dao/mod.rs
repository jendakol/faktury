use std::convert::TryFrom;
use std::future::Future;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

use diesel::mysql::MysqlConnection;
use diesel::prelude::*;
use diesel::sql_types;
use diesel::{insert_into, select};
use err_context::AnyError;
use log::{debug, warn};

use crate::config::DbConfig;
use crate::dao::models::{Contact, Entrepreneur};

mod models;
mod schema;

embed_migrations!("migrations");

// TODO metrics

pub type DaoResult<A> = Result<A, AnyError>;

#[derive(Clone)]
pub struct Dao {
    connection: Arc<Mutex<MysqlConnection>>,
}

impl TryFrom<DbConfig> for Dao {
    type Error = AnyError;

    fn try_from(config: DbConfig) -> Result<Self, Self::Error> {
        let database_url = format!(
            "mysql://{}:{}@{}:{}/{}?prefer_socket={}",
            config.username,
            config.password,
            config.host,
            config.port,
            config.db_name,
            config.prefer_socket
        );

        // TODO connection pooling?

        debug!("Connecting to MySQL @ {}:{}", config.host, config.port);

        let connection = MysqlConnection::establish(&database_url)
            .map_err(|e| AnyError::from(format!("Error connecting to {}: {}", database_url, e)))?;

        if let Err(f) = embedded_migrations::run(&connection) {
            warn!("Error while migrating the DB: {}", f)
        }

        let connection = Arc::new(Mutex::new(connection));

        Ok(Dao { connection })
    }
}

no_arg_sql_function!(last_insert_id, sql_types::Integer);

impl Dao {
    fn with_connection<F, R>(&self, f: F) -> impl Future<Output = R>
    where
        F: FnOnce(&MysqlConnection) -> R,
    {
        let lock = self.connection.clone();

        futures::future::lazy(move |_| {
            let conn = lock.lock().unwrap();
            let conn: &MysqlConnection = conn.deref();

            f(conn)
        })
    }

    fn last_inserted_id(conn: &MysqlConnection) -> DaoResult<i32> {
        select(last_insert_id).first(conn).map_err(AnyError::from)
    }

    fn map_db_error(err: diesel::result::Error) -> AnyError {
        AnyError::from(format!("DB error: {}", err))
    }

    pub async fn load_contact(&self) -> Contact {
        use schema::contacts::dsl::*;

        let results = self
            .with_connection(|conn| contacts.load::<Contact>(conn).unwrap())
            .await;

        results.first().unwrap().clone()
    }

    pub async fn insert_entrepreneur(
        &self,
        code: &str,
        name: &str,
        addr: &str,
    ) -> DaoResult<Entrepreneur> {
        let id = self
            .with_connection(|conn| {
                use schema::entrepreneurs::dsl as table;

                insert_into(table::entrepreneurs)
                    .values((
                        table::code.eq(code),
                        table::name.eq(name),
                        table::address.eq(addr),
                    ))
                    .execute(conn)
                    .map_err(Self::map_db_error)
                    .and_then(|r| {
                        if r == 1 {
                            Self::last_inserted_id(conn)
                        } else if r == 0 {
                            Err(AnyError::from("No inserted row!"))
                        } else {
                            debug!("Weird - more than 1 affected row!");
                            Err(AnyError::from("More than 1 inserted row!"))
                        }
                    })
            })
            .await?;

        Ok(Entrepreneur {
            id,
            code: code.to_owned(),
            name: name.to_owned(),
            address: addr.to_owned(),
        })
    }

    pub async fn insert_contact(
        &self,
        ent_id: i32,
        code: &str,
        name: &str,
        addr: &str,
    ) -> DaoResult<Contact> {
        let id = self
            .with_connection(|conn| {
                use schema::contacts::dsl as table;

                insert_into(table::contacts)
                    .values((
                        table::code.eq(code),
                        table::entrepreneur_id.eq(ent_id),
                        table::name.eq(name),
                        table::address.eq(addr),
                    ))
                    .execute(conn)
                    .map_err(Self::map_db_error)
                    .and_then(|r| {
                        if r == 1 {
                            Self::last_inserted_id(conn)
                        } else if r == 0 {
                            Err(AnyError::from("No inserted row!"))
                        } else {
                            debug!("Weird - more than 1 affected row!");
                            Err(AnyError::from("More than 1 inserted row!"))
                        }
                    })
            })
            .await?;

        Ok(Contact {
            id,
            entrepreneur_id: ent_id,
            code: code.to_owned(),
            name: name.to_owned(),
            address: addr.to_owned(),
        })
    }
}
