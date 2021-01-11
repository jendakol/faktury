use actix_web::web::Bytes;
use err_context::AnyError;
use log::{debug, warn};

use pdf::PdfManager;
use settings::AccountSettings;

use crate::dao::{Account, Dao, DaoResult, Entrepreneur, Invoice, InvoiceWithAllInfo};
use crate::handlers::dto::NewInvoice;
use crate::logic::invoices as InvoicesLogic;

pub mod auth;
pub mod iban;
pub mod invoices;
pub mod pdf;
pub mod settings;

pub async fn download_invoice(
    dao: &Dao,
    pdf_manager: &PdfManager,
    id: u32,
) -> Result<(Invoice, impl futures::Stream<Item = Result<Bytes, ()>>), AnyError> {
    let (invoice, rows) = match dao.get_invoice_with_rows(id).await? {
        Some(iwr) => iwr,
        None => return Err(AnyError::from("Could not find requested invoice")),
    };

    let entrepreneur = dao
        .get_entrepreneur(invoice.entrepreneur_id as u32)
        .await?
        .expect("This value must exist!");

    let account = dao
        .get_account(entrepreneur.account_id as u32)
        .await?
        .expect("This value must exist!");

    let contact = dao.get_contact(invoice.contact_id as u32).await?.expect("This value must exist!");

    let account_settings = AccountSettings::from(&account);

    Ok((
        invoice.clone(),
        pdf_manager.create(account_settings, entrepreneur, contact, invoice, rows),
    ))
}

pub async fn insert_invoice(dao: &Dao, invoice: &NewInvoice) -> DaoResult<InvoiceWithAllInfo> {
    let entrepreneur = dao
        .get_entrepreneur(invoice.entrepreneur_id as u32)
        .await?
        .expect("This value must exist!");

    let account = dao
        .get_account(entrepreneur.account_id as u32)
        .await?
        .expect("This value must exist!");

    let settings = AccountSettings::from(&account);

    debug!("Loaded user settings: {:?}", settings);

    let invoice_code = next_invoice_code(dao, &entrepreneur, &account, &settings).await?;

    // TODO handle duplicated code

    dao.insert_invoice(
        &invoice_code,
        invoice.entrepreneur_id,
        invoice.contact_id,
        &settings.invoice.default_due_length,
    )
    .await
}

pub async fn copy_invoice(dao: &Dao, original: Invoice) -> Result<Invoice, AnyError> {
    let entrepreneur = dao
        .get_entrepreneur(original.entrepreneur_id as u32)
        .await?
        .expect("This value must exist!");

    let account = dao
        .get_account(entrepreneur.account_id as u32)
        .await?
        .expect("This value must exist!");

    let settings = AccountSettings::from(&account);

    debug!("Loaded user settings: {:?}", settings);

    let invoice_code = next_invoice_code(dao, &entrepreneur, &account, &settings).await?;

    let (copy, _, _) = dao
        .insert_invoice(
            &invoice_code,
            original.entrepreneur_id as u32,
            original.contact_id as u32,
            &settings.invoice.default_due_length,
        )
        .await?;

    let original_rows = dao.get_invoice_rows(original.id as u32).await?;

    for row in original_rows {
        let _ = dao
            .insert_invoice_row(copy.id as u32, &row.item_name, row.item_price, row.item_count as u16)
            .await;
    }

    Ok(copy)
}

async fn next_invoice_code(
    dao: &Dao,
    entrepreneur: &Entrepreneur,
    account: &Account,
    settings: &AccountSettings,
) -> Result<String, AnyError> {
    InvoicesLogic::next_code(dao, &account, &entrepreneur, settings.invoice.naming_schema)
        .await
        .map_err(|err| {
            warn!("Could not generate invoice id: {}", err);
            AnyError::from("Could not generate invoice id")
        })
}
