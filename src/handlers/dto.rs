use chrono::NaiveDateTime as Datetime;
use frunk::LabelledGeneric;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, LabelledGeneric, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Contact {
    pub id: i32,
    pub code: String,
    pub entrepreneur_id: i32,
    pub name: String,
    pub address: String,
}

#[derive(Serialize, Deserialize, LabelledGeneric, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Entrepreneur {
    pub id: i32,
    pub code: String,
    pub name: String,
    pub address: String,
}

#[derive(Serialize, Deserialize, LabelledGeneric, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Invoice {
    pub id: i32,
    pub code: String,
    pub entrepreneur_id: i32,
    pub contact_id: i32,
    pub created: Datetime,
    pub pay_until: Datetime,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payed: Option<Datetime>,
}

#[derive(Serialize, Deserialize, LabelledGeneric, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceRow {
    pub id: i32,
    pub invoice_id: i32,
    pub item_name: String,
    pub item_price: i32,
    pub item_count: i8,
}

// ******

impl From<crate::dao::Entrepreneur> for Entrepreneur {
    fn from(i: crate::dao::Entrepreneur) -> Self {
        frunk::labelled_convert_from(i)
    }
}

impl From<crate::dao::Contact> for Contact {
    fn from(i: crate::dao::Contact) -> Self {
        frunk::labelled_convert_from(i)
    }
}

impl From<crate::dao::Invoice> for Invoice {
    fn from(i: crate::dao::Invoice) -> Self {
        frunk::labelled_convert_from(i)
    }
}

impl From<crate::dao::InvoiceRow> for InvoiceRow {
    fn from(i: crate::dao::InvoiceRow) -> Self {
        frunk::labelled_convert_from(i)
    }
}
