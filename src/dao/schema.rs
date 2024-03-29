table! {
    accounts (id) {
        id -> Integer,
        username -> Varchar,
        salt -> Varchar,
        password -> Varchar,
        settings -> Text,
    }
}

table! {
    contacts (id) {
        id -> Integer,
        entrepreneur_id -> Integer,
        code -> Nullable<Varchar>,
        name -> Varchar,
        address -> Varchar,
        vat -> Varchar,
    }
}

table! {
    entrepreneurs (id) {
        id -> Integer,
        account_id -> Integer,
        code -> Varchar,
        name -> Varchar,
        address -> Varchar,
        vat -> Varchar,
        account_number_country_code -> Varchar,
        account_number_prefix -> Nullable<Smallint>,
        account_number -> Bigint,
        account_bank_code -> Smallint,
        email -> Nullable<Varchar>,
        phone -> Nullable<Varchar>,
        currency_code -> Varchar,
    }
}

table! {
    invoices (id) {
        id -> Integer,
        entrepreneur_id -> Integer,
        contact_id -> Integer,
        code -> Varchar,
        created -> Date,
        pay_until -> Date,
        payed -> Nullable<Date>,
    }
}

table! {
    invoice_rows (id) {
        id -> Integer,
        invoice_id -> Integer,
        item_name -> Varchar,
        item_price -> Float,
        item_count -> SmallInt,
    }
}

table! {
    login_sessions (id) {
        id -> VarChar,
        account_id -> Integer,
    }
}

joinable!(contacts -> entrepreneurs (entrepreneur_id));
joinable!(entrepreneurs -> accounts (account_id));
joinable!(invoice_rows -> invoices (invoice_id));
joinable!(invoices -> contacts (contact_id));
joinable!(invoices -> entrepreneurs (entrepreneur_id));
joinable!(login_sessions -> accounts (account_id));

allow_tables_to_appear_in_same_query!(accounts, contacts, entrepreneurs, invoices, invoice_rows,);
