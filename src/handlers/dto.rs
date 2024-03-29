use std::collections::HashMap;

use chrono::NaiveDate as Date;
use frunk::*;
use serde::{Deserialize, Serialize};

use crate::dao::MonthlyMoney;
use crate::dao::Vat;

#[derive(Serialize, Deserialize, LabelledGeneric, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub id: i32,
    pub username: String,
    pub settings: String,
}

#[derive(Serialize, Deserialize, LabelledGeneric, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Contact {
    pub id: i32,
    pub entrepreneur_id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    pub name: String,
    pub address: String,
    pub vat: Vat,
}

#[derive(Serialize, Deserialize, LabelledGeneric, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Entrepreneur {
    pub id: i32,
    pub account_id: i32,
    pub code: String,
    pub name: String,
    pub address: String,
    pub vat: Vat,
    pub account_number_country_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_number_prefix: Option<i16>,
    pub account_number: i64,
    pub account_bank_code: i16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    pub currency_code: String,
}

#[derive(Serialize, Deserialize, LabelledGeneric, Generic, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Invoice {
    pub id: i32,
    pub entrepreneur_id: i32,
    pub contact_id: i32,
    pub code: String,
    pub created: Date,
    pub pay_until: Date,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payed: Option<Date>,
}

#[derive(Serialize, Deserialize, LabelledGeneric, Generic, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceWithAllInfo {
    pub id: i32,
    pub entrepreneur_id: i32,
    pub contact_id: i32,
    pub code: String,
    pub created: Date,
    pub pay_until: Date,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payed: Option<Date>,
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
    pub item_count: i16,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceWithRows {
    pub invoice: Invoice,
    pub rows: Vec<InvoiceRow>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LoginSession {
    pub id: String,
    pub account_id: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LoginSessionCreated {
    pub encoded_value: String,
    pub ttl: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct InvoicesListParams {
    pub last: Option<u16>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ContactsListParams {
    pub count: Option<u16>,
    pub last_months: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MonthlyStat {
    pub paid: f64,
    pub unpaid: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct YearlyStats(HashMap<u8, MonthlyStat>);

// ****** New*:

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NewContact {
    pub code: Option<String>,
    pub entrepreneur_id: u32,
    pub name: String,
    pub address: String,
    pub vat: Vat,
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
    pub account_id: u32,
    pub entrepreneur_id: u32,
    pub contact_id: u32,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NewInvoiceRow {
    pub invoice_id: u32,
    pub item_name: String,
    pub item_price: f32,
    pub item_count: u16,
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

impl From<crate::dao::InvoiceWithAllInfo> for InvoiceWithAllInfo {
    fn from(i: crate::dao::InvoiceWithAllInfo) -> Self {
        let (invoice, price_sum, contact_name) = i;
        let inv_repr = frunk::into_generic(invoice);
        let inv_repr = inv_repr + hlist![price_sum, contact_name];
        frunk::from_generic(inv_repr)
    }
}

impl From<crate::dao::InvoiceRow> for InvoiceRow {
    fn from(i: crate::dao::InvoiceRow) -> Self {
        frunk::labelled_convert_from(i)
    }
}

impl From<crate::dao::LoginSession> for LoginSession {
    fn from(s: crate::dao::LoginSession) -> Self {
        LoginSession {
            id: s.id,
            account_id: s.account_id as u32,
        }
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

impl From<(Vec<MonthlyMoney>, Vec<MonthlyMoney>)> for YearlyStats {
    fn from(d: (Vec<MonthlyMoney>, Vec<MonthlyMoney>)) -> Self {
        let (paid, unpaid) = d;

        // paid.sort_by(|a, b| Ord::cmp(&a.month, &b.month));
        // unpaid.sort_by(|a, b| Ord::cmp(&a.month, &b.month));

        let paid = paid.into_iter().map(|s| (s.month as u8, s.money)).collect::<HashMap<u8, f64>>();
        let unpaid = unpaid.into_iter().map(|s| (s.month as u8, s.money)).collect::<HashMap<u8, f64>>();

        let mut stats = HashMap::new();

        for month in 1..=12u8 {
            let paid_m = paid.get(&month).copied().unwrap_or(0f64);
            let unpaid_m = unpaid.get(&month).copied().unwrap_or(0f64);

            stats.insert(
                month,
                MonthlyStat {
                    paid: paid_m,
                    unpaid: unpaid_m,
                },
            );
        }

        YearlyStats(stats)
    }
}
