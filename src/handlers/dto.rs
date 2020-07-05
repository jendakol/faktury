use chrono::NaiveDateTime as Datetime;
use frunk::*;
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

#[derive(Serialize, Deserialize, LabelledGeneric, Generic, Debug, Clone)]
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

#[derive(Serialize, Deserialize, LabelledGeneric, Generic, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceWithAllInfo {
    pub id: i32,
    pub code: String,
    pub entrepreneur_id: i32,
    pub contact_id: i32,
    pub created: Datetime,
    pub pay_until: Datetime,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payed: Option<Datetime>,
    pub price_sum: f64,
    pub contact_name: String,
}

#[derive(Serialize, Deserialize, LabelledGeneric, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceRow {
    pub id: i32,
    pub invoice_id: i32,
    pub item_name: String,
    pub item_price: f32,
    pub item_count: i8,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceWithRows {
    pub invoice: Invoice,
    pub rows: Vec<InvoiceRow>,
}

// ****** New*:

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NewContact {
    pub code: String,
    pub entrepreneur_id: u32,
    pub name: String,
    pub address: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NewEntrepreneur {
    pub code: String,
    pub name: String,
    pub address: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NewInvoice {
    pub code: String,
    pub entrepreneur_id: u32,
    pub contact_id: u32,
    pub created: Datetime,
    pub pay_until: Datetime,
    pub payed: Option<Datetime>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NewInvoiceRow {
    pub invoice_id: u32,
    pub item_name: String,
    pub item_price: f32,
    pub item_count: u8,
}

// ******

// TODO do this with macro:

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

// impl From<crate::dao::InvoiceWithAllInfo> for InvoiceWithAllInfo {
//     fn from(i: crate::dao::InvoiceWithAllInfo) -> Self {
//         frunk::labelled_convert_from(i)
//     }
// }

impl From<crate::dao::InvoiceWithAllInfo> for InvoiceWithAllInfo {
    fn from(i: crate::dao::InvoiceWithAllInfo) -> Self {
        let (invoice, price_sum, contact_name) = i;
        let inv_repr = frunk::into_generic(invoice);
        let inv_repr = inv_repr + hlist![price_sum, contact_name];
        frunk::from_generic(inv_repr)
    }
}
//
// impl From<crate::dao::InvoiceWithPrice> for Invoice {
//     fn from(i: crate::dao::InvoiceWithPrice) -> Self {
//         let (invoice, price_sum) = i;
//         let inv_repr = frunk::into_generic(invoice);
//         let inv_repr = inv_repr + hlist![Some(price_sum)];
//         frunk::from_generic(inv_repr)
//     }
// }

impl From<crate::dao::InvoiceRow> for InvoiceRow {
    fn from(i: crate::dao::InvoiceRow) -> Self {
        frunk::labelled_convert_from(i)
    }
}

impl From<Entrepreneur> for crate::dao::Entrepreneur {
    fn from(i: Entrepreneur) -> Self {
        frunk::labelled_convert_from(i)
    }
}

impl From<Contact> for crate::dao::Contact {
    fn from(i: Contact) -> Self {
        frunk::labelled_convert_from(i)
    }
}

impl From<Invoice> for crate::dao::Invoice {
    fn from(i: Invoice) -> Self {
        frunk::labelled_convert_from(i)
    }
}

impl From<InvoiceRow> for crate::dao::InvoiceRow {
    fn from(i: InvoiceRow) -> Self {
        frunk::labelled_convert_from(i)
    }
}
