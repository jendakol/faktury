table! {
    contacts (id) {
        id -> Integer,
        code -> Varchar,
        name -> Varchar,
        address -> Varchar,
    }
}

table! {
    invoices (id) {
        id -> Integer,
        code -> Varchar,
        contact_id -> Integer,
        created -> Datetime,
        pay_until -> Datetime,
        payed -> Nullable<Datetime>,
    }
}

table! {
    invoice_rows (id) {
        id -> Integer,
        invoice_id -> Integer,
        item_name -> Varchar,
        item_price -> Float,
        item_count -> Tinyint,
    }
}

joinable!(invoice_rows -> invoices (invoice_id));
joinable!(invoices -> contacts (contact_id));

allow_tables_to_appear_in_same_query!(
    contacts,
    invoices,
    invoice_rows,
);
