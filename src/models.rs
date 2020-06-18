use super::schema::*;
use chrono::NaiveDateTime as Datetime;

#[derive(Debug, Queryable)]
pub struct Contact {
    pub id: i32,
    pub code: String,
    pub name: String,
    pub address: String,
}

#[derive(Debug, Insertable)]
#[table_name = "contacts"]
pub struct NewContact<'a> {
    pub code: &'a str,
    pub name: &'a str,
    pub address: &'a str,
}

#[derive(Debug, Queryable)]
pub struct Invoice {
    pub id: i32,
    pub contact_id: i32,
    pub created: Datetime,
    pub pay_until: Datetime,
    pub payed: Datetime,
}

#[derive(Debug, Insertable)]
#[table_name = "invoices"]
pub struct NewInvoice {
    pub contact_id: i32,
    pub created: Datetime,
    pub pay_until: Datetime,
    pub payed: Datetime,
}

#[derive(Debug, Queryable)]
pub struct InvoiceRow {
    pub id: i32,
    pub invoice_id: i32,
    pub item_name: String,
    pub item_price: i32,
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
