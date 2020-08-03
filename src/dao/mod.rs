use std::convert::TryFrom;
use std::future::Future;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

use chrono::{Duration, NaiveDateTime as DateTime};
use diesel::dsl::max;
use diesel::mysql::MysqlConnection;
use diesel::prelude::*;
use diesel::{delete, insert_into, select};
use diesel::{sql_types, update};
use diesel_logger::LoggingConnection;
use err_context::AnyError;
use log::{debug, warn};

use crate::config::DbConfig;
use crate::dao::models::NewInvoice;
pub use crate::dao::models::{Account, Contact, Entrepreneur, Invoice, InvoiceRow};

mod models;
mod schema;

embed_migrations!("migrations");

// TODO metrics

pub type DaoResult<A> = Result<A, AnyError>;
pub type InvoiceWithAllInfo = (Invoice, f64, String);

#[derive(Clone)]
pub struct Dao {
    connection: Arc<Mutex<LoggingConnection<MysqlConnection>>>,
}

impl TryFrom<DbConfig> for Dao {
    type Error = AnyError;

    fn try_from(config: DbConfig) -> Result<Self, Self::Error> {
        let database_url = format!(
            "mysql://{}:{}@{}:{}/{}?prefer_socket={}",
            config.username, config.password, config.host, config.port, config.db_name, config.prefer_socket
        );

        // TODO connection pooling?

        debug!("Connecting to MySQL @ {}:{}", config.host, config.port);

        let connection = MysqlConnection::establish(&database_url)
            .map_err(|e| AnyError::from(format!("Error connecting to {}: {}", database_url, e)))?;

        if let Err(f) = embedded_migrations::run(&connection) {
            warn!("Error while migrating the DB: {}", f)
        }

        let connection = LoggingConnection::new(connection);
        let connection = Arc::new(Mutex::new(connection));

        Ok(Dao { connection })
    }
}

no_arg_sql_function!(last_insert_id, sql_types::Integer);

// TODO macro for select_single?

impl Dao {
    // *** GET SINGLE:

    pub async fn get_account(&self, id: u32) -> DaoResult<Option<Account>> {
        use schema::accounts::dsl as table;

        self.with_connection(|conn| table::accounts.filter(table::id.eq(id as i32)).first(conn).optional())
            .await
            .map_err(Self::map_db_error)
    }

    pub async fn get_entrepreneur(&self, id: u32) -> DaoResult<Option<Entrepreneur>> {
        use schema::entrepreneurs::dsl as table;

        self.with_connection(|conn| table::entrepreneurs.filter(table::id.eq(id as i32)).first(conn).optional())
            .await
            .map_err(Self::map_db_error)
    }

    pub async fn get_invoice(&self, id: u32) -> DaoResult<Option<InvoiceWithAllInfo>> {
        use schema::*;

        self.with_connection(|conn| {
            invoices::table
                .select((
                    invoices::all_columns,
                    diesel::dsl::sql::<diesel::sql_types::Double>(
                        "ifnull((select sum(invoice_rows.item_price) from invoice_rows where invoice_rows.invoice_id=invoices.id), 0)",
                    ),
                    diesel::dsl::sql::<diesel::sql_types::VarChar>(
                        "(select contacts.name from contacts where contacts.id=invoices.contact_id)",
                    ),
                ))
                .filter(invoices::id.eq(id as i32))
                .first(conn)
                .optional()
        })
        .await
        .map_err(Self::map_db_error)
    }

    pub async fn get_invoice_with_rows(&self, id: u32) -> DaoResult<Option<(Invoice, Vec<InvoiceRow>)>> {
        use schema::*;

        let results: Vec<(Invoice, Option<InvoiceRow>)> = self
            .with_connection(|conn| {
                invoices::table
                    .left_outer_join(invoice_rows::table)
                    .filter(invoices::id.eq(id as i32))
                    .load(conn)
            })
            .await
            .map_err(Self::map_db_error)?;

        let mut invoice_opt = None;
        let mut rows = Vec::new();

        results.into_iter().for_each(|(invoice, row)| {
            if invoice_opt.is_none() {
                invoice_opt = Some(invoice);
            }

            if let Some(row) = row {
                rows.push(row)
            }
        });

        Ok(invoice_opt.map(|inv| (inv, rows)))
    }

    pub async fn get_invoice_row(&self, id: u32) -> DaoResult<Option<InvoiceRow>> {
        use schema::invoice_rows::dsl as table;

        self.with_connection(|conn| table::invoice_rows.filter(table::id.eq(id as i32)).first(conn).optional())
            .await
            .map_err(Self::map_db_error)
    }

    pub async fn get_contact(&self, id: u32) -> DaoResult<Option<Contact>> {
        use schema::contacts::dsl as table;

        self.with_connection(|conn| table::contacts.filter(table::id.eq(id as i32)).first(conn).optional())
            .await
            .map_err(Self::map_db_error)
    }

    // *** GET LIST:

    pub async fn get_contacts(&self, entrepreneur_id: u32) -> DaoResult<Vec<Contact>> {
        use schema::contacts::dsl as table;

        self.with_connection(|conn| table::contacts.filter(table::entrepreneur_id.eq(entrepreneur_id as i32)).load(conn))
            .await
            .map_err(Self::map_db_error)
    }

    pub async fn get_invoices(&self, entrepreneur_id: u32) -> DaoResult<Vec<InvoiceWithAllInfo>> {
        use schema::*;

        self.with_connection(|conn| {
            invoices::table
                .select((
                    invoices::all_columns,
                    // This is not exactly nice and type-safe piece of code. However, I'm unable to convince Diesel to create it by his own - I just don't know how. 
                    diesel::dsl::sql::<diesel::sql_types::Double>("ifnull((select sum(invoice_rows.item_price * invoice_rows.item_count) from invoice_rows where invoice_rows.invoice_id=invoices.id), 0)"),
                    diesel::dsl::sql::<diesel::sql_types::VarChar>("(select contacts.name from contacts where contacts.id=invoices.contact_id)"),
                ))
                .filter(invoices::entrepreneur_id.eq(entrepreneur_id as i32))
                .load(conn)
        })
            .await
            .map_err(Self::map_db_error)
    }

    pub async fn get_invoice_rows(&self, invoice_id: u32) -> DaoResult<Vec<InvoiceRow>> {
        use schema::invoice_rows::dsl as table;

        self.with_connection(|conn| table::invoice_rows.filter(table::invoice_id.eq(invoice_id as i32)).load(conn))
            .await
            .map_err(Self::map_db_error)
    }

    // *** INSERT:

    pub async fn insert_entrepreneur(&self, code: &str, name: &str, addr: &str) -> DaoResult<Entrepreneur> {
        let id = self
            .with_connection(|conn| {
                use schema::entrepreneurs::dsl as table;

                insert_into(table::entrepreneurs)
                    .values((table::code.eq(code), table::name.eq(name), table::address.eq(addr)))
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

    pub async fn insert_contact(&self, ent_id: u32, code: &str, name: &str, addr: &str) -> DaoResult<Contact> {
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

        Ok(self.get_contact(id as u32).await?.expect("Must find newly inserted contact!"))
    }

    pub async fn insert_invoice(
        &self,
        code: &str,
        entrepreneur_id: u32,
        contact_id: u32,
        due_length: &Duration,
    ) -> DaoResult<InvoiceWithAllInfo> {
        let id = self
            .with_connection(|conn| {
                use schema::invoices::dsl as table;

                let now: DateTime = select(diesel::dsl::now).get_result::<DateTime>(conn)?;

                let pay_until = now + *due_length;
                let pay_until = pay_until.date();

                let invoice = NewInvoice {
                    entrepreneur_id: entrepreneur_id as i32,
                    contact_id: contact_id as i32,
                    code,
                    created: now,
                    pay_until,
                    payed: None,
                };

                debug!("Inserting new invoice: {:?}", invoice);

                insert_into(table::invoices)
                    .values(invoice)
                    .execute(conn)
                    .map_err(Self::map_db_error)
                    .and_then(|r| Self::get_new_id(conn, r))
            })
            .await?; // it's already mapped to DB error

        Ok(self.get_invoice(id as u32).await?.expect("Must find newly inserted invoice!"))
    }

    pub async fn insert_invoice_row(&self, invoice_id: u32, name: &str, price: f32, count: u8) -> DaoResult<InvoiceRow> {
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

    // *** OTHERS:

    pub async fn get_invoices_max_id(&self, ent_id: u32) -> DaoResult<i32> {
        self.with_connection(|conn| {
            use schema::*;

            invoices::table
                .select(max(invoices::id))
                .filter(invoices::entrepreneur_id.eq(ent_id as i32))
                .first::<Option<i32>>(conn)
                .map_err(Self::map_db_error)
                .map(|r| r.unwrap_or(0)) // no at all, default to 0
        })
        .await
    }

    // *** HELPER METHODS:

    fn with_connection<F, R>(&self, f: F) -> impl Future<Output = R>
    where
        F: FnOnce(&LoggingConnection<MysqlConnection>) -> R,
    {
        let lock = self.connection.clone();

        futures::future::lazy(move |_| {
            let conn = lock.lock().unwrap();
            let conn = conn.deref();

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
        select(last_insert_id).first(conn).map_err(Self::map_db_error)
    }

    fn map_db_error(err: diesel::result::Error) -> AnyError {
        AnyError::from(format!("DB error: {}", err))
    }
}
