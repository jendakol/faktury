use chrono::NaiveDate as Date;
use frunk::{Generic, LabelledGeneric};

use crate::dao::Vat;

use super::schema::*;

#[derive(Identifiable, Queryable, QueryableByName, Associations, AsChangeset, LabelledGeneric, PartialEq, Debug, Clone)]
#[table_name = "accounts"]
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

#[derive(Identifiable, Queryable, QueryableByName, Associations, AsChangeset, LabelledGeneric, PartialEq, Debug, Clone)]
#[belongs_to(Account)]
#[table_name = "entrepreneurs"]
#[changeset_options(treat_none_as_null = "true")]
pub struct Entrepreneur {
    pub id: i32,
    pub account_id: i32,
    pub code: String,
    pub name: String,
    pub address: String,
    pub vat: Vat,
    pub account_number_country_code: String,
    pub account_number_prefix: Option<i16>,
    pub account_number: i64,
    pub account_bank_code: i16,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub currency_code: String,
}

#[derive(Debug, Insertable)]
#[table_name = "entrepreneurs"]
pub struct NewEntrepreneur<'a> {
    pub code: &'a str,
    pub name: &'a str,
    pub address: &'a str,
    pub vat: Vat,
    pub account_number_country_code: &'a str,
    pub account_number_prefix: Option<i16>,
    pub account_number: i64,
    pub account_bank_code: i16,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub currency_code: String,
}

#[derive(Identifiable, Queryable, QueryableByName, Associations, AsChangeset, LabelledGeneric, PartialEq, Debug, Clone)]
#[belongs_to(Entrepreneur)]
#[table_name = "contacts"]
#[changeset_options(treat_none_as_null = "true")]
pub struct Contact {
    pub id: i32,
    pub entrepreneur_id: i32,
    pub code: Option<String>,
    pub name: String,
    pub address: String,
    pub vat: Vat,
}

#[derive(Debug, Insertable)]
#[table_name = "contacts"]
pub struct NewContact<'a> {
    pub entrepreneur_id: i32,
    pub code: Option<&'a str>,
    pub name: &'a str,
    pub address: &'a str,
    pub vat: Option<Vat>,
}

#[derive(Identifiable, Queryable, QueryableByName, Associations, AsChangeset, LabelledGeneric, Generic, PartialEq, Debug, Clone)]
#[belongs_to(Entrepreneur)]
#[table_name = "invoices"]
#[changeset_options(treat_none_as_null = "true")]
pub struct Invoice {
    pub id: i32,
    pub entrepreneur_id: i32,
    pub contact_id: i32,
    pub code: String,
    pub created: Date,
    pub pay_until: Date,
    pub payed: Option<Date>,
}

#[derive(Debug, Insertable)]
#[table_name = "invoices"]
pub struct NewInvoice<'a> {
    pub entrepreneur_id: i32,
    pub contact_id: i32,
    pub code: &'a str,
    pub created: Option<Date>,
    pub pay_until: Date,
    pub payed: Option<Date>,
}

#[derive(Identifiable, Queryable, QueryableByName, Associations, AsChangeset, LabelledGeneric, PartialEq, Debug, Clone)]
#[belongs_to(Invoice)]
#[table_name = "invoice_rows"]
#[changeset_options(treat_none_as_null = "true")]
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

#[derive(Identifiable, Queryable, QueryableByName, Associations, AsChangeset, LabelledGeneric, PartialEq, Debug, Clone)]
#[belongs_to(Account)]
#[table_name = "login_sessions"]
#[changeset_options(treat_none_as_null = "true")]
pub struct LoginSession {
    pub id: String,
    pub account_id: i32,
}

#[derive(Debug, Insertable)]
#[table_name = "login_sessions"]
pub struct NewSession<'a> {
    pub id: &'a str,
    pub account_id: i32,
}
