use std::future::Future;
use std::ops::Deref;

use actix_web::body::BodyStream;
use actix_web::{get, post, web, HttpResponse, Responder};
use log::{debug, warn};

use crate::dao::DaoResult;
use crate::handlers::dto::{
    Contact, Entrepreneur, Invoice, InvoiceRow, InvoiceWithRows, NewContact, NewEntrepreneur,
    NewInvoice, NewInvoiceRow,
};
use crate::logic;
use crate::logic::invoices as InvoicesLogic;
use crate::logic::settings::AccountSettings;
use crate::RequestContext;

mod dto;

#[get("/download/{id}")]
pub async fn download_invoice(
    id: web::Path<u32>,
    ctx: web::Data<RequestContext>,
) -> impl Responder {
    match logic::download_invoice(id.into_inner(), &ctx.dao, &ctx.pdf_manager).await {
        Ok(stream) => HttpResponse::Ok().body(BodyStream::new(stream)),
        Err(err) => {
            warn!("Error while downloading invoice PDF: {}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}

// TODO security: login and https://docs.rs/csrf/0.4.0/csrf/

#[post("/data-get/entrepreneur/{id}")]
pub async fn get_entrepreneur(
    entrepreneur_id: web::Path<u32>,
    ctx: web::Data<RequestContext>,
) -> impl Responder {
    debug!("Getting entrepreneur data, ID {}", entrepreneur_id);

    with_found(
        ctx.dao.get_entrepreneur(entrepreneur_id.into_inner()),
        |i| async { HttpResponse::Ok().json::<dto::Entrepreneur>(i.into()) },
    )
    .await
}

#[post("/data-get/invoice/{id}")]
pub async fn get_invoice(
    invoice_id: web::Path<u32>,
    ctx: web::Data<RequestContext>,
) -> impl Responder {
    debug!("Getting invoice data, ID {}", invoice_id);

    with_found(ctx.dao.get_invoice(invoice_id.into_inner()), |i| async {
        HttpResponse::Ok().json::<dto::InvoiceWithAllInfo>(i.into())
    })
    .await
}

#[post("/data-get/invoice-with-rows/{id}")]
pub async fn get_invoice_with_rows(
    invoice_id: web::Path<u32>,
    ctx: web::Data<RequestContext>,
) -> impl Responder {
    debug!("Getting invoice data incl. rows, ID {}", invoice_id);

    with_found(
        ctx.dao.get_invoice_with_rows(invoice_id.into_inner()),
        |(invoice, rows)| async {
            let iwr = InvoiceWithRows {
                invoice: invoice.into(),
                rows: rows.into_iter().map(|r| r.into()).collect(),
            };

            HttpResponse::Ok().json(iwr)
        },
    )
    .await
}

#[post("/data-get/contact/{id}")]
pub async fn get_contact(
    contact_id: web::Path<u32>,
    ctx: web::Data<RequestContext>,
) -> impl Responder {
    debug!("Getting contact data, ID {}", contact_id);

    with_found(ctx.dao.get_contact(contact_id.into_inner()), |i| async {
        HttpResponse::Ok().json::<dto::Contact>(i.into())
    })
    .await
}

#[post("/data-get/contacts/{id}")]
pub async fn list_contacts(
    ent_id: web::Path<u32>,
    ctx: web::Data<RequestContext>,
) -> impl Responder {
    debug!("Getting contacts list for entrepreneur ID {}", ent_id);

    with_ok(ctx.dao.get_contacts(ent_id.into_inner()), |rows| async {
        HttpResponse::Ok().json::<Vec<dto::Contact>>(rows.into_iter().map(|r| r.into()).collect())
    })
    .await
}

#[post("/data-get/invoices/{id}")]
pub async fn list_invoices(
    ent_id: web::Path<u32>,
    ctx: web::Data<RequestContext>,
) -> impl Responder {
    debug!("Getting invoices list for entrepreneur ID {}", ent_id);

    with_ok(ctx.dao.get_invoices(ent_id.into_inner()), |rows| async {
        HttpResponse::Ok()
            .json::<Vec<dto::InvoiceWithAllInfo>>(rows.into_iter().map(|r| r.into()).collect())
    })
    .await
}

#[post("/data-get/invoice-rows/{id}")]
pub async fn list_invoice_rows(
    invoice_id: web::Path<u32>,
    ctx: web::Data<RequestContext>,
) -> impl Responder {
    debug!("Getting invoice rows data, ID {}", invoice_id);

    with_ok(
        ctx.dao.get_invoice_rows(invoice_id.into_inner()),
        |rows| async {
            HttpResponse::Ok()
                .json::<Vec<dto::InvoiceRow>>(rows.into_iter().map(|r| r.into()).collect())
        },
    )
    .await
}

#[post("/data-insert/entrepreneur")]
pub async fn insert_entrepreneur(
    entrepreneur: web::Json<NewEntrepreneur>,
    ctx: web::Data<RequestContext>,
) -> impl Responder {
    debug!("Inserting new entrepreneur: {:?}", entrepreneur);

    with_ok(
        ctx.dao.insert_entrepreneur(
            &entrepreneur.code,
            &entrepreneur.name,
            &entrepreneur.address,
        ),
        |i| async { HttpResponse::Ok().json::<dto::Entrepreneur>(i.into()) },
    )
    .await
}

#[post("/data-insert/contact")]
pub async fn insert_contact(
    contact: web::Json<NewContact>,
    ctx: web::Data<RequestContext>,
) -> impl Responder {
    debug!("Inserting new contact: {:?}", contact);

    with_ok(
        ctx.dao.insert_contact(
            contact.entrepreneur_id,
            &contact.code,
            &contact.name,
            &contact.address,
        ),
        |i| async { HttpResponse::Ok().json::<dto::Contact>(i.into()) },
    )
    .await
}

#[post("/data-insert/invoice")]
pub async fn insert_invoice(
    invoice: web::Json<NewInvoice>,
    ctx: web::Data<RequestContext>,
) -> impl Responder {
    use InvoicesLogic::next_code;

    debug!("Inserting new invoice: {:?}", invoice);

    with_found(ctx.dao.get_account(invoice.account_id), |account| async {
        let settings = AccountSettings::from(&account);

        debug!("Loaded user settings: {:?}", settings);

        let invoice_code = match next_code(&ctx.dao, settings.invoice.naming_schema).await {
            Ok(code) => code,
            Err(err) => {
                warn!("Could not generate invoice id: {}", err);
                return HttpResponse::InternalServerError().body("Could not generate invoice id");
            }
        };

        drop(account); // TODO WTF

        with_ok(
            ctx.dao.insert_invoice(
                &invoice_code,
                invoice.entrepreneur_id,
                invoice.contact_id,
                settings.invoice.default_due_length.deref(),
            ),
            |i| async { HttpResponse::Ok().json::<dto::InvoiceWithAllInfo>(i.into()) },
        )
        .await
    })
    .await
}

#[post("/data-insert/invoice-row")]
pub async fn insert_invoice_row(
    row: web::Json<NewInvoiceRow>,
    ctx: web::Data<RequestContext>,
) -> impl Responder {
    debug!("Inserting new invoice row: {:?}", row);

    with_ok(
        ctx.dao.insert_invoice_row(
            row.invoice_id,
            &row.item_name,
            row.item_price,
            row.item_count,
        ),
        |i| async { HttpResponse::Ok().json::<dto::InvoiceRow>(i.into()) },
    )
    .await
}

#[post("/data-update/entrepreneur")]
pub async fn update_entrepreneur(
    entrepreneur: web::Json<Entrepreneur>,
    ctx: web::Data<RequestContext>,
) -> impl Responder {
    debug!("Updating entrepreneur: {:?}", entrepreneur);

    with_ok(
        ctx.dao
            .update_entrepreneur(&entrepreneur.into_inner().into()),
        |_| async { HttpResponse::Ok().body("{\"success\":true}") },
    )
    .await
}

#[post("/data-update/contact")]
pub async fn update_contact(
    contact: web::Json<Contact>,
    ctx: web::Data<RequestContext>,
) -> impl Responder {
    debug!("Updating contact: {:?}", contact);

    with_ok(
        ctx.dao.update_contact(&contact.into_inner().into()),
        |_| async { HttpResponse::Ok().body("{\"success\":true}") },
    )
    .await
}

#[post("/data-update/invoice")]
pub async fn update_invoice(
    invoice: web::Json<Invoice>,
    ctx: web::Data<RequestContext>,
) -> impl Responder {
    debug!("Deleting invoice: {:?}", invoice);

    with_ok(
        ctx.dao.update_invoice(&invoice.into_inner().into()),
        |_| async { HttpResponse::Ok().body("{\"success\":true}") },
    )
    .await
}

#[post("/data-update/invoice-row")]
pub async fn update_invoice_row(
    row: web::Json<InvoiceRow>,
    ctx: web::Data<RequestContext>,
) -> impl Responder {
    debug!("Deleting invoice row: {:?}", row);

    with_ok(
        ctx.dao.update_invoice_row(&row.into_inner().into()),
        |_| async { HttpResponse::Ok().body("{\"success\":true}") },
    )
    .await
}

#[post("/data-delete/entrepreneur/{id}")]
pub async fn delete_entrepreneur(
    id: web::Path<u32>,
    ctx: web::Data<RequestContext>,
) -> impl Responder {
    debug!("Deleting entrepreneur ID {}", id);

    with_ok(ctx.dao.delete_entrepreneur(id.into_inner()), |_| async {
        HttpResponse::Ok().body("{\"success\":true}")
    })
    .await
}

#[post("/data-delete/contact/{id}")]
pub async fn delete_contact(id: web::Path<u32>, ctx: web::Data<RequestContext>) -> impl Responder {
    debug!("Deleting contact ID {}", id);

    with_ok(ctx.dao.delete_contact(id.into_inner()), |_| async {
        HttpResponse::Ok().body("{\"success\":true}")
    })
    .await
}

#[post("/data-delete/invoice/{id}")]
pub async fn delete_invoice(id: web::Path<u32>, ctx: web::Data<RequestContext>) -> impl Responder {
    debug!("Updating invoice ID {}", id);

    with_ok(ctx.dao.delete_invoice(id.into_inner()), |_| async {
        HttpResponse::Ok().body("{\"success\":true}")
    })
    .await
}

#[post("/data-delete/invoice-row/{id}")]
pub async fn delete_invoice_row(
    id: web::Path<u32>,
    ctx: web::Data<RequestContext>,
) -> impl Responder {
    debug!("Updating invoice row ID {}", id);

    with_ok(ctx.dao.delete_invoice_row(id.into_inner()), |_| async {
        HttpResponse::Ok().body("{\"success\":true}")
    })
    .await
}

async fn with_ok<A, F, Fu>(req: impl Future<Output = DaoResult<A>>, f: F) -> HttpResponse
where
    Fu: Future<Output = HttpResponse>,
    F: FnOnce(A) -> Fu,
{
    match req.await {
        Ok(a) => f(a).await,
        Err(e) => {
            warn!("Error while querying DB: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

async fn with_found<A, F, Fu>(req: impl Future<Output = DaoResult<Option<A>>>, f: F) -> HttpResponse
where
    Fu: Future<Output = HttpResponse>,
    F: FnOnce(A) -> Fu,
{
    with_ok(req, |r| async {
        match r {
            Some(a) => f(a).await,
            None => HttpResponse::NotFound().finish(),
        }
    })
    .await
}
