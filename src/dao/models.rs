use chrono::NaiveDateTime as Datetime;
use frunk::{Generic, LabelledGeneric};

use super::schema::*;

#[derive(
    Identifiable, Queryable, Associations, AsChangeset, LabelledGeneric, PartialEq, Debug, Clone,
)]
pub struct Entrepreneur {
    pub id: i32,
    pub code: String,
    pub name: String,
    pub address: String,
}

#[derive(Debug, Insertable)]
#[table_name = "entrepreneurs"]
pub struct NewEntrepreneur<'a> {
    pub code: &'a str,
    pub name: &'a str,
    pub address: &'a str,
}

#[derive(
    Identifiable, Queryable, Associations, AsChangeset, LabelledGeneric, PartialEq, Debug, Clone,
)]
#[belongs_to(Entrepreneur)]
pub struct Contact {
    pub id: i32,
    pub code: String,
    pub entrepreneur_id: i32,
    pub name: String,
    pub address: String,
}

#[derive(Debug, Insertable)]
#[table_name = "contacts"]
pub struct NewContact<'a> {
    pub code: &'a str,
    pub entrepreneur_id: i32,
    pub name: &'a str,
    pub address: &'a str,
}

#[derive(
    Identifiable,
    Queryable,
    Associations,
    AsChangeset,
    LabelledGeneric,
    Generic,
    PartialEq,
    Debug,
    Clone,
)]
#[belongs_to(Entrepreneur)]
pub struct Invoice {
    pub id: i32,
    pub code: String,
    pub entrepreneur_id: i32,
    pub contact_id: i32,
    pub created: Datetime,
    pub pay_until: Datetime,
    pub payed: Option<Datetime>,
}

#[derive(Debug, Insertable)]
#[table_name = "invoices"]
pub struct NewInvoice {
    pub code: String,
    pub entrepreneur_id: i32,
    pub contact_id: i32,
    pub created: Datetime,
    pub pay_until: Datetime,
    pub payed: Option<Datetime>,
}

#[derive(
    Identifiable, Queryable, Associations, AsChangeset, LabelledGeneric, PartialEq, Debug, Clone,
)]
#[belongs_to(Invoice)]
pub struct InvoiceRow {
    pub id: i32,
    pub invoice_id: i32,
    pub item_name: String,
    pub item_price: f32,
    pub item_count: i8,
}

#[derive(Debug, Insertable)]
#[table_name = "invoice_rows"]
pub struct NewInvoiceRow {
    pub invoice_id: i32,
    pub item_name: String,
    pub item_price: f32,
    pub item_count: i8,
}
