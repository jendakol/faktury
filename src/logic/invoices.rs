use err_context::AnyError;

pub use inner::InvoiceNamingSchemaType;
use inner::*;

use crate::dao::{Account, Dao, Entrepreneur};

pub async fn next_code(
    dao: &Dao,
    account: &Account,
    entrepreneur: &Entrepreneur,
    naming_type: InvoiceNamingSchemaType,
) -> Result<String, AnyError> {
    match naming_type {
        InvoiceNamingSchemaType::Default => DefaultInvoiceNaming::next_code(dao, account, entrepreneur).await,
    }
}

mod inner {
    use async_trait::async_trait;
    use chrono::{Datelike, Local};
    use err_context::AnyError;
    use log::debug;
    use serde::{Deserialize, Serialize};

    use crate::dao::{Account, Dao, Entrepreneur};

    #[async_trait]
    pub trait InvoiceNamingSchema {
        async fn next_code(dao: &Dao, account: &Account, entrepreneur: &Entrepreneur) -> Result<String, AnyError>;
    }

    #[derive(Serialize, Deserialize, Copy, Clone, Debug)]
    pub enum InvoiceNamingSchemaType {
        Default,
    }

    impl Default for InvoiceNamingSchemaType {
        fn default() -> Self {
            InvoiceNamingSchemaType::Default
        }
    }

    pub(super) struct DefaultInvoiceNaming;

    #[async_trait]
    impl InvoiceNamingSchema for DefaultInvoiceNaming {
        async fn next_code(dao: &Dao, _account: &Account, entrepreneur: &Entrepreneur) -> Result<String, AnyError> {
            let max_id = dao.get_invoices_max_id(entrepreneur).await?;
            let next_id = max_id + 1;

            let now = Local::now();

            // TODO next_id % 100?

            let code = format!("{}{:02}{:02}", now.year(), now.month(), next_id);

            debug!("Generated code for invoice: {}", code);

            Ok(code)
        }
    }
}
