use async_trait::async_trait;
use diesel::sql_types::Bool;
use diesel::RunQueryDsl;

use crate::dao::Dao;
use crate::handlers::LoginSession;

#[async_trait]
pub trait Auth {
    async fn is_valid_for_invoice(&self, dao: &Dao, invoice_id: u32) -> bool;
    async fn is_valid_for_invoice_row(&self, dao: &Dao, row_id: u32) -> bool;
    async fn is_valid_for_entrepreneur(&self, dao: &Dao, entrepreneur_id: u32) -> bool;
    async fn is_valid_for_contact(&self, dao: &Dao, contact_id: u32) -> bool;
}

/// This struct exists because Diesel doesn't allow to return tuples from raw queries:
/// https://docs.diesel.rs/diesel/fn.sql_query.html
#[derive(QueryableByName)]
struct ValidationResult {
    #[sql_type = "Bool"]
    pub result: bool,
}

#[async_trait]
impl Auth for LoginSession {
    async fn is_valid_for_invoice(&self, dao: &Dao, invoice_id: u32) -> bool {
        let sql = format!(
            r#"SELECT entrepreneurs.account_id = {} as result FROM invoices
                join entrepreneurs on entrepreneurs.id=invoices.entrepreneur_id
                where invoices.id={}"#,
            self.account_id, invoice_id
        );

        is_valid_for(dao, sql).await
    }

    async fn is_valid_for_invoice_row(&self, dao: &Dao, row_id: u32) -> bool {
        let sql = format!(
            r#"SELECT entrepreneurs.account_id = {} as result FROM invoice_rows
                join invoices on invoices.id=invoice_rows.invoice_id
                join entrepreneurs on entrepreneurs.id=invoices.entrepreneur_id
                where invoice_rows.id={}"#,
            self.account_id, row_id
        );

        is_valid_for(dao, sql).await
    }

    async fn is_valid_for_entrepreneur(&self, dao: &Dao, entrepreneur_id: u32) -> bool {
        let sql = format!(
            "SELECT entrepreneurs.account_id = {} as result FROM entrepreneurs where entrepreneurs.id={}",
            self.account_id, entrepreneur_id
        );

        is_valid_for(dao, sql).await
    }

    async fn is_valid_for_contact(&self, dao: &Dao, contact_id: u32) -> bool {
        let sql = format!(
            r#"SELECT entrepreneurs.account_id = {} as result FROM contacts
                join entrepreneurs on entrepreneurs.id=contacts.entrepreneur_id
                where contacts.id={}"#,
            self.account_id, contact_id
        );

        is_valid_for(dao, sql).await
    }
}

async fn is_valid_for(dao: &Dao, sql: String) -> bool {
    dao.with_connection(|conn| diesel::sql_query(sql).load::<ValidationResult>(conn))
        .await
        .expect("Unable to check session rights")
        .first()
        .expect("Returned empty set of results where it shouldn't be possible")
        .result
}
