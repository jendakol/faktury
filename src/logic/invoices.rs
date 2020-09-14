use err_context::AnyError;

pub use inner::InvoiceNamingSchemaType;
use inner::*;

use crate::dao::Dao;

pub async fn next_code(dao: &Dao, naming_type: InvoiceNamingSchemaType) -> Result<String, AnyError> {
    match naming_type {
        InvoiceNamingSchemaType::Default => DefaultInvoiceNaming::next_code(dao).await,
    }
}

mod inner {
    use async_trait::async_trait;
    use chrono::{Datelike, Local};
    use err_context::AnyError;
    use serde::{Deserialize, Serialize};

    use crate::dao::Dao;

    #[async_trait]
    pub trait InvoiceNamingSchema {
        async fn next_code(dao: &Dao) -> Result<String, AnyError>;
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
        async fn next_code(dao: &Dao) -> Result<String, AnyError> {
            let max_id = dao.get_invoices_max_id(1).await?;
            let next_id = max_id + 1;

            let now = Local::now();

            // TODO make the format better

            Ok(format!("{}{:02}{:02}{}", now.year(), now.month(), now.day(), next_id))
        }
    }
}
