table! {
    contacts (id) {
        id -> Integer,
        code -> Varchar,
        entrepreneur_id -> Integer,
        name -> Varchar,
        address -> Varchar,
    }
}

table! {
    entrepreneurs (id) {
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
        entrepreneur_id -> Integer,
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

joinable!(contacts -> entrepreneurs (entrepreneur_id));
joinable!(invoice_rows -> invoices (invoice_id));
joinable!(invoices -> contacts (contact_id));
joinable!(invoices -> entrepreneurs (entrepreneur_id));

allow_tables_to_appear_in_same_query!(
    contacts,
    entrepreneurs,
    invoices,
    invoice_rows,
);
