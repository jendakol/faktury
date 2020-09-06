use actix_web::web::Bytes;
use err_context::AnyError;

use pdf::PdfManager;
use settings::AccountSettings;

use crate::dao::{Dao, Invoice};

pub mod iban;
pub mod invoices;
pub mod pdf;
pub mod settings;

pub async fn download_invoice(
    id: u32,
    dao: &Dao,
    pdf_manager: &PdfManager,
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
