use std::convert::TryFrom;
use std::future::Future;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

use chrono::NaiveDateTime as Datetime;
use diesel::mysql::MysqlConnection;
use diesel::prelude::*;
use diesel::{delete, insert_into, select};
use diesel::{sql_types, update};
use err_context::AnyError;
use log::{debug, warn};

use crate::config::DbConfig;
pub use crate::dao::models::{Contact, Entrepreneur, Invoice, InvoiceRow};

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
    // *** GET SINGLE:

    pub async fn get_entrepreneur(&self, id: u32) -> DaoResult<Option<Entrepreneur>> {
        use schema::entrepreneurs::dsl as table;

        let mut results = self
            .with_connection(|conn| {
                table::entrepreneurs
                    .filter(table::id.eq(id as i32))
                    .limit(1)
                    .load(conn)
            })
            .await
            .map_err(Self::map_db_error)?;

        Ok(results.pop())
    }

    pub async fn get_invoice(&self, id: u32) -> DaoResult<Option<Invoice>> {
        use schema::invoices::dsl as table;

        let mut results: Vec<_> = self
            .with_connection(|conn| {
                table::invoices
                    .filter(table::id.eq(id as i32))
                    .limit(1)
                    .load(conn)
            })
            .await
            .map_err(Self::map_db_error)?;

        Ok(results.pop())
    }

    pub async fn get_invoice_row(&self, id: u32) -> DaoResult<Option<InvoiceRow>> {
        use schema::invoice_rows::dsl as table;

        let mut results: Vec<_> = self
            .with_connection(|conn| {
                table::invoice_rows
                    .filter(table::id.eq(id as i32))
                    .limit(1)
                    .load(conn)
            })
            .await
            .map_err(Self::map_db_error)?;

        Ok(results.pop())
    }

    pub async fn get_contact(&self, id: u32) -> DaoResult<Option<Contact>> {
        use schema::contacts::dsl as table;

        let mut results = self
            .with_connection(|conn| {
                table::contacts
                    .filter(table::id.eq(id as i32))
                    .limit(1)
                    .load(conn)
            })
            .await
            .map_err(Self::map_db_error)?;

        Ok(results.pop())
    }

    // *** GET LIST:

    pub async fn get_contacts(&self, entrepreneur_id: u32) -> DaoResult<Vec<Contact>> {
        use schema::contacts::dsl as table;

        self.with_connection(|conn| {
            table::contacts
                .filter(table::entrepreneur_id.eq(entrepreneur_id as i32))
                .load(conn)
        })
        .await
        .map_err(Self::map_db_error)
    }

    pub async fn get_invoices(&self, entrepreneur_id: u32) -> DaoResult<Vec<Invoice>> {
        use schema::invoices::dsl as table;

        self.with_connection(|conn| {
            table::invoices
                .filter(table::entrepreneur_id.eq(entrepreneur_id as i32))
                .load(conn)
        })
        .await
        .map_err(Self::map_db_error)
    }

    pub async fn get_invoice_rows(&self, invoice_id: u32) -> DaoResult<Vec<InvoiceRow>> {
        use schema::invoice_rows::dsl as table;

        self.with_connection(|conn| {
            table::invoice_rows
                .filter(table::invoice_id.eq(invoice_id as i32))
                .load(conn)
        })
        .await
        .map_err(Self::map_db_error)
    }

    // *** INSERT:

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
                    .and_then(|r| Self::get_new_id(conn, r))
            })
            .await?; // it's already mapped to DB error

        Ok(self
            .get_entrepreneur(id as u32)
            .await?
            .expect("Must find newly inserted entrepreneur!"))
    }

    pub async fn insert_contact(
        &self,
        ent_id: u32,
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
                        table::entrepreneur_id.eq(ent_id as i32),
                        table::name.eq(name),
                        table::address.eq(addr),
                    ))
                    .execute(conn)
                    .map_err(Self::map_db_error)
                    .and_then(|r| Self::get_new_id(conn, r))
            })
            .await?; // it's already mapped to DB error

        Ok(self
            .get_contact(id as u32)
            .await?
            .expect("Must find newly inserted contact!"))
    }

    pub async fn insert_invoice(
        &self,
        code: &str,
        ent_id: u32,
        cont_id: u32,
        pay_until: Datetime,
    ) -> DaoResult<Invoice> {
        let id = self
            .with_connection(|conn| {
                use schema::invoices::dsl as table;

                insert_into(table::invoices)
                    .values((
                        table::code.eq(code),
                        table::entrepreneur_id.eq(ent_id as i32),
                        table::contact_id.eq(cont_id as i32),
                        table::pay_until.eq(pay_until),
                    ))
                    .execute(conn)
                    .map_err(Self::map_db_error)
                    .and_then(|r| Self::get_new_id(conn, r))
            })
            .await?; // it's already mapped to DB error

        Ok(self
            .get_invoice(id as u32)
            .await?
            .expect("Must find newly inserted invoice!"))
    }

    pub async fn insert_invoice_row(
        &self,
        invoice_id: u32,
        name: &str,
        price: f32,
        count: u8,
    ) -> DaoResult<InvoiceRow> {
        let id = self
            .with_connection(|conn| {
                use schema::invoice_rows::dsl as table;

                insert_into(table::invoice_rows)
                    .values((
                        table::invoice_id.eq(invoice_id as i32),
                        table::item_name.eq(name),
                        table::item_price.eq(price),
                        table::item_count.eq(count as i8),
                    ))
                    .execute(conn)
                    .map_err(Self::map_db_error)
                    .and_then(|r| Self::get_new_id(conn, r))
            })
            .await?; // it's already mapped to DB error

        Ok(self
            .get_invoice_row(id as u32)
            .await?
            .expect("Must find newly inserted invoice row!"))
    }

    // *** UPDATE:

    pub async fn update_entrepreneur(&self, ent: &Entrepreneur) -> DaoResult<()> {
        self.with_connection(|conn| {
            use schema::entrepreneurs::dsl as table;

            update(table::entrepreneurs)
                .set(ent)
                .filter(table::id.eq(ent.id))
                .execute(conn)
                .map_err(Self::map_db_error)
        })
        .await?; // it's already mapped to DB error

        Ok(())
    }

    pub async fn update_contact(&self, contact: &Contact) -> DaoResult<()> {
        self.with_connection(|conn| {
            use schema::contacts::dsl as table;

            update(table::contacts)
                .set(contact)
                .filter(table::id.eq(contact.id))
                .execute(conn)
                .map_err(Self::map_db_error)
        })
        .await?; // it's already mapped to DB error

        Ok(())
    }

    pub async fn update_invoice(&self, invoice: &Invoice) -> DaoResult<()> {
        self.with_connection(|conn| {
            use schema::invoices::dsl as table;

            update(table::invoices)
                .set(invoice)
                .filter(table::id.eq(invoice.id))
                .execute(conn)
                .map_err(Self::map_db_error)
        })
        .await?; // it's already mapped to DB error

        Ok(())
    }

    pub async fn update_invoice_row(&self, invoice_row: &InvoiceRow) -> DaoResult<()> {
        self.with_connection(|conn| {
            use schema::invoice_rows::dsl as table;

            update(table::invoice_rows)
                .set(invoice_row)
                .filter(table::id.eq(invoice_row.id))
                .execute(conn)
                .map_err(Self::map_db_error)
        })
        .await?; // it's already mapped to DB error

        Ok(())
    }

    // *** DELETE:

    pub async fn delete_entrepreneur(&self, id: u32) -> DaoResult<()> {
        self.with_connection(|conn| {
            use schema::entrepreneurs::dsl as table;

            delete(table::entrepreneurs)
                .filter(table::id.eq(id as i32))
                .execute(conn)
                .map_err(Self::map_db_error)
        })
        .await?; // it's already mapped to DB error

        Ok(())
    }

    pub async fn delete_contact(&self, id: u32) -> DaoResult<()> {
        self.with_connection(|conn| {
            use schema::contacts::dsl as table;

            delete(table::contacts)
                .filter(table::id.eq(id as i32))
                .execute(conn)
                .map_err(Self::map_db_error)
        })
        .await?; // it's already mapped to DB error

        Ok(())
    }

    pub async fn delete_invoice(&self, id: u32) -> DaoResult<()> {
        self.with_connection(|conn| {
            use schema::invoices::dsl as table;

            delete(table::invoices)
                .filter(table::id.eq(id as i32))
                .execute(conn)
                .map_err(Self::map_db_error)
        })
        .await?; // it's already mapped to DB error

        Ok(())
    }

    pub async fn delete_invoice_row(&self, id: u32) -> DaoResult<()> {
        self.with_connection(|conn| {
            use schema::invoice_rows::dsl as table;

            delete(table::invoice_rows)
                .filter(table::id.eq(id as i32))
                .execute(conn)
                .map_err(Self::map_db_error)
        })
        .await?; // it's already mapped to DB error

        Ok(())
    }

    // *** HELPER METHODS:

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

    fn get_new_id(conn: &MysqlConnection, r: usize) -> Result<i32, AnyError> {
        if r == 1 {
            Self::last_inserted_id(conn)
        } else if r == 0 {
            Err(AnyError::from("No inserted row!"))
        } else {
            debug!("Weird - more than 1 affected row!");
            Err(AnyError::from("More than 1 inserted row!"))
        }
    }

    fn last_inserted_id(conn: &MysqlConnection) -> DaoResult<i32> {
        select(last_insert_id)
            .first(conn)
            .map_err(Self::map_db_error)
    }

    fn map_db_error(err: diesel::result::Error) -> AnyError {
        AnyError::from(format!("DB error: {}", err))
    }
}
