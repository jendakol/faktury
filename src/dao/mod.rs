use std::convert::TryFrom;
use std::future::Future;
use std::io::Write;
use std::sync::{Arc, Mutex};

use chrono::{Duration, NaiveDateTime as DateTime};
use diesel::backend::Backend;
use diesel::deserialize::FromSql;
use diesel::expression::dsl::count;
use diesel::mysql::MysqlConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::serialize::{Output, ToSql};
use diesel::sql_query;
use diesel::sql_types::VarChar;
use diesel::{delete, deserialize, insert_into, select, serialize};
use diesel::{sql_types, update};
use diesel_logger::LoggingConnection;
use err_context::AnyError;
use log::{debug, warn};
use serde::{Deserialize, Serialize};

use crate::config::DbConfig;
use crate::dao::models::NewInvoice;
pub use crate::dao::models::{Account, Contact, Entrepreneur, Invoice, InvoiceRow, LoginSession};

mod models;
mod schema;

embed_migrations!("migrations");

// TODO metrics

pub type DaoResult<A> = Result<A, AnyError>;
pub type InvoiceWithAllInfo = (Invoice, f64, String);

type MysqlConnectionManager = ConnectionManager<LoggingConnection<MysqlConnection>>;
type MysqlPool = Pool<MysqlConnectionManager>;
type MysqlPooledConnection = PooledConnection<MysqlConnectionManager>;

#[derive(Debug, Serialize, Deserialize, FromSqlRow, AsExpression, PartialEq, Clone)]
#[sql_type = "VarChar"]
pub enum Vat {
    Code(String),
    NotTaxPayer,
    DontDisplay,
}

impl<DB> FromSql<VarChar, DB> for Vat
where
    DB: Backend,
    String: FromSql<VarChar, DB>,
{
    fn from_sql(bytes: Option<&DB::RawValue>) -> deserialize::Result<Self> {
        Ok(serde_json::from_str(&String::from_sql(bytes)?)?)
    }
}

impl<DB> ToSql<VarChar, DB> for Vat
where
    DB: Backend,
    String: ToSql<VarChar, DB>,
{
    fn to_sql<W: Write>(&self, out: &mut Output<W, DB>) -> serialize::Result {
        serde_json::to_string(self)?.to_sql(out)
    }
}

#[derive(Clone)]
pub struct Dao {
    pool: Arc<Mutex<MysqlPool>>,
}

impl TryFrom<DbConfig> for Dao {
    type Error = AnyError;

    fn try_from(config: DbConfig) -> Result<Self, Self::Error> {
        let database_url = format!(
            "mysql://{}:{}@{}:{}/{}?prefer_socket={}",
            config.username, config.password, config.host, config.port, config.db_name, config.prefer_socket
        );

        debug!("Connecting to MySQL @ {}:{}", config.host, config.port);

        let manager: MysqlConnectionManager = ConnectionManager::new(&database_url);

        let pool = Pool::builder()
            .max_size(config.max_pool_size as u32)
            .build(manager)
            .map_err(|e| AnyError::from(format!("Error connecting to {}: {}", database_url, e)))?;

        let connection: MysqlPooledConnection = pool
            .get()
            .map_err(|e| AnyError::from(format!("Error connecting to {}: {}", database_url, e)))?;

        if let Err(f) = embedded_migrations::run(&connection) {
            warn!("Error while migrating the DB: {}", f)
        }

        let pool = Arc::new(Mutex::new(pool));

        Ok(Dao { pool })
    }
}

no_arg_sql_function!(last_insert_id, sql_types::Integer);

// TODO macro for select_single?

impl Dao {
    // *** ACCOUNT:

    pub async fn find_account(&self, username: &str, password: &str) -> DaoResult<Option<Account>> {
        use schema::accounts::dsl as table;

        self.with_connection(|conn| {
            table::accounts
                .filter(table::username.eq(username))
                .filter(table::password.eq(password))
                .first(conn)
                .optional()
        })
        .await
        .map_err(Self::map_db_error)
    }

    pub async fn new_session(&self, account: &Account) -> DaoResult<LoginSession> {
        use schema::login_sessions::dsl as table;

        let session_id = uuid::Uuid::new_v4().to_string();

        self.with_connection(|conn| {
            insert_into(table::login_sessions)
                .values((table::id.eq(&session_id), table::account_id.eq(account.id)))
                .execute(conn)
                .map_err(Self::map_db_error)
        })
        .await?;

        Ok(self
            .get_session(&session_id)
            .await?
            .expect("Must find newly inserted login session!"))
    }

    pub async fn find_session(&self, id: &str) -> DaoResult<Option<LoginSession>> {
        use schema::login_sessions::dsl as table;

        self.with_connection(|conn| table::login_sessions.filter(table::id.eq(id)).first(conn).optional())
            .await
            .map_err(Self::map_db_error)
    }

    pub async fn revoke_session(&self, session_id: String) -> DaoResult<()> {
        use schema::login_sessions::dsl as table;

        self.with_connection(|conn| {
            delete(table::login_sessions)
                .filter(table::id.eq(session_id))
                .execute(conn)
                .map_err(Self::map_db_error)
        })
        .await?;

        Ok(())
    }

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

    pub async fn get_session(&self, id: &str) -> DaoResult<Option<LoginSession>> {
        use schema::login_sessions::dsl as table;

        self.with_connection(|conn| table::login_sessions.filter(table::id.eq(id)).first(conn).optional())
            .await
            .map_err(Self::map_db_error)
    }

    // *** GET LIST:

    pub async fn get_entrepreneurs(&self, account_id: u32) -> DaoResult<Vec<Entrepreneur>> {
        use schema::entrepreneurs::dsl as table;

        self.with_connection(|conn| table::entrepreneurs.filter(table::account_id.eq(account_id as i32)).load(conn))
            .await
            .map_err(Self::map_db_error)
    }

    pub async fn get_contacts(&self, entrepreneur_id: u32, limit: Option<u16>, last_months: Option<u8>) -> DaoResult<Vec<Contact>> {
        // Here I'm not patient enough to convince Diesel to construct the right query :-( Sorryfor that.

        let mut sql = format!("select * from contacts where entrepreneur_id = {}", entrepreneur_id);

        if let Some(m) = last_months {
            sql += &format!(" order by (select count(id) from invoices where invoices.contact_id = contacts.id and invoices.created >= DATE_SUB(NOW(),INTERVAL {} MONTH)) desc", m);
        } else {
            sql += "order by name asc";
        }

        if let Some(c) = limit {
            sql += &format!(" limit {}", c);
        }

        self.with_connection(|conn| sql_query(sql).load(conn))
            .await
            .map_err(Self::map_db_error)
    }

    pub async fn get_invoices(&self, entrepreneur_id: u32, limit: Option<u16>) -> DaoResult<Vec<InvoiceWithAllInfo>> {
        use schema::*;

        self.with_connection(|conn| {
            let query = invoices::table
                .select((
                    invoices::all_columns,
                    // This is not exactly nice and type-safe piece of code. However, I'm unable to convince Diesel to create it by his own - I just don't know how.
                    diesel::dsl::sql::<diesel::sql_types::Double>("ifnull((select sum(invoice_rows.item_price * invoice_rows.item_count) from invoice_rows where invoice_rows.invoice_id=invoices.id), 0)"),
                    diesel::dsl::sql::<diesel::sql_types::VarChar>("(select contacts.name from contacts where contacts.id=invoices.contact_id)"),
                ))
                .filter(invoices::entrepreneur_id.eq(entrepreneur_id as i32))
                .order(invoices::created.desc());

            if let Some(c) = limit {
                query.limit(c as i64).load(conn)
            } else {
                query.load(conn)
            }
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

    pub async fn insert_contact(&self, ent_id: u32, code: &Option<String>, name: &str, addr: &str, vat: &Vat) -> DaoResult<Contact> {
        let id = self
            .with_connection(|conn| {
                use schema::contacts::dsl as table;

                // TODO null instead of empty code
                insert_into(table::contacts)
                    .values((
                        table::code.eq(code),
                        table::entrepreneur_id.eq(ent_id as i32),
                        table::name.eq(name),
                        table::address.eq(addr),
                        table::vat.eq(vat),
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

    pub async fn insert_invoice_row(&self, invoice_id: u32, name: &str, price: f32, count: u16) -> DaoResult<InvoiceRow> {
        let id = self
            .with_connection(|conn| {
                use schema::invoice_rows::dsl as table;

                insert_into(table::invoice_rows)
                    .values((
                        table::invoice_id.eq(invoice_id as i32),
                        table::item_name.eq(name),
                        table::item_price.eq(price),
                        table::item_count.eq(count as i16),
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
                .set((
                    table::code.eq(&contact.code),
                    table::name.eq(&contact.name),
                    table::address.eq(&contact.address),
                    table::vat.eq(&contact.vat),
                ))
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

    pub async fn get_invoices_max_id(&self, entrepreneur: &Entrepreneur) -> DaoResult<u32> {
        self.with_connection(|conn| {
            use schema::*;

            invoices::table
                .select(count(invoices::id))
                .filter(invoices::entrepreneur_id.eq(entrepreneur.id))
                .first::<i64>(conn)
                .map(|c| c as u32) // this looks dangerous, but I believe there won't be more invoices than 2^32...
                .map_err(Self::map_db_error)
        })
        .await
    }

    // *** HELPER METHODS:

    pub fn with_connection<F, R>(&self, f: F) -> impl Future<Output = R>
    where
        F: FnOnce(&MysqlConnection) -> R,
    {
        let lock = self.pool.clone();

        futures::future::lazy(move |_| {
            let conn: MysqlPooledConnection = {
                let pool = lock.lock().expect("Could not get connection pool mutex lock");
                pool.get().expect("Coul")
            };

            f(&conn)
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
