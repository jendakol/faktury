use chrono::NaiveDate as Date;
use chrono::NaiveDateTime as Datetime;
use frunk::{Generic, LabelledGeneric};

use super::schema::*;

#[derive(Identifiable, Queryable, Associations, AsChangeset, LabelledGeneric, PartialEq, Debug, Clone)]
pub struct Account {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub settings: String,
}

#[derive(Debug, Insertable)]
#[table_name = "accounts"]
pub struct NewAccount<'a> {
    pub username: &'a str,
    pub password: &'a str,
    pub settings: &'a str,
}

#[derive(Identifiable, Queryable, Associations, AsChangeset, LabelledGeneric, PartialEq, Debug, Clone)]
#[belongs_to(Account)]
pub struct Entrepreneur {
    pub id: i32,
    pub account_id: i32,
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

#[derive(Identifiable, Queryable, Associations, AsChangeset, LabelledGeneric, PartialEq, Debug, Clone)]
#[belongs_to(Entrepreneur)]
pub struct Contact {
    pub id: i32,
    pub entrepreneur_id: i32,
    pub code: Option<String>,
    pub name: String,
    pub address: String,
}

#[derive(Debug, Insertable)]
#[table_name = "contacts"]
pub struct NewContact<'a> {
    pub entrepreneur_id: i32,
    pub code: Option<&'a str>,
    pub name: &'a str,
    pub address: &'a str,
}

#[derive(Identifiable, Queryable, Associations, AsChangeset, LabelledGeneric, Generic, PartialEq, Debug, Clone)]
#[belongs_to(Entrepreneur)]
pub struct Invoice {
    pub id: i32,
    pub entrepreneur_id: i32,
    pub contact_id: i32,
    pub code: String,
    pub created: Datetime,
    pub pay_until: Date,
    pub payed: Option<Date>,
}

#[derive(Debug, Insertable)]
#[table_name = "invoices"]
pub struct NewInvoice<'a> {
    pub entrepreneur_id: i32,
    pub contact_id: i32,
    pub code: &'a str,
    pub created: Datetime,
    pub pay_until: Date,
    pub payed: Option<Date>,
}

#[derive(Identifiable, Queryable, Associations, AsChangeset, LabelledGeneric, PartialEq, Debug, Clone)]
#[belongs_to(Invoice)]
pub struct InvoiceRow {
    pub id: i32,
    pub invoice_id: i32,
    pub item_name: String,
    pub item_price: f32,
    pub item_count: i16,
}

#[derive(Debug, Insertable)]
#[table_name = "invoice_rows"]
pub struct NewInvoiceRow {
    pub invoice_id: i32,
    pub item_name: String,
    pub item_price: f32,
    pub item_count: i16,
}
