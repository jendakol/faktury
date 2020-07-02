use std::future::Future;

use actix_web::body::BodyStream;
use actix_web::{get, post, web, HttpResponse, Responder};
use log::{debug, warn};

use crate::dao::DaoResult;
use crate::handlers::dto::{
    Contact, Entrepreneur, Invoice, InvoiceRow, NewContact, NewEntrepreneur, NewInvoice,
    NewInvoiceRow,
};
use crate::RequestContext;

mod dto;

#[get("/download")]
pub async fn get(ctx: web::Data<RequestContext>) -> impl Responder {
    with_found(ctx.dao.get_contact(1), |contact| {
        let pdf_stream = ctx.pdf_manager.create(contact.name);
        HttpResponse::Ok().body(BodyStream::new(pdf_stream))
    })
    .await
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
        |i| HttpResponse::Ok().json::<dto::Entrepreneur>(i.into()),
    )
    .await
}

#[post("/data-get/invoice/{id}")]
pub async fn get_invoice(
    invoice_id: web::Path<u32>,
    ctx: web::Data<RequestContext>,
) -> impl Responder {
    debug!("Getting invoice data, ID {}", invoice_id);

    with_found(ctx.dao.get_invoice(invoice_id.into_inner()), |i| {
        HttpResponse::Ok().json::<dto::Invoice>(i.into())
    })
    .await
}

#[post("/data-get/contact/{id}")]
pub async fn get_contact(
    contact_id: web::Path<u32>,
    ctx: web::Data<RequestContext>,
) -> impl Responder {
    debug!("Getting contact data, ID {}", contact_id);

    with_found(ctx.dao.get_contact(contact_id.into_inner()), |i| {
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

    with_ok(ctx.dao.get_contacts(ent_id.into_inner()), |rows| {
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

    with_ok(ctx.dao.get_invoices(ent_id.into_inner()), |rows| {
        HttpResponse::Ok().json::<Vec<dto::Invoice>>(rows.into_iter().map(|r| r.into()).collect())
    })
    .await
}

#[post("/data-get/invoice-rows/{id}")]
pub async fn list_invoice_rows(
    invoice_id: web::Path<u32>,
    ctx: web::Data<RequestContext>,
) -> impl Responder {
    debug!("Getting invoice rows data, ID {}", invoice_id);

    with_ok(ctx.dao.get_invoice_rows(invoice_id.into_inner()), |rows| {
        HttpResponse::Ok()
            .json::<Vec<dto::InvoiceRow>>(rows.into_iter().map(|r| r.into()).collect())
    })
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
        |i| HttpResponse::Ok().json::<dto::Entrepreneur>(i.into()),
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
        |i| HttpResponse::Ok().json::<dto::Contact>(i.into()),
    )
    .await
}

#[post("/data-insert/invoice")]
pub async fn insert_invoice(
    invoice: web::Json<NewInvoice>,
    ctx: web::Data<RequestContext>,
) -> impl Responder {
    debug!("Inserting new invoice: {:?}", invoice);

    with_ok(
        ctx.dao.insert_invoice(
            &invoice.code,
            invoice.entrepreneur_id,
            invoice.contact_id,
            invoice.pay_until,
        ),
        |i| HttpResponse::Ok().json::<dto::Invoice>(i.into()),
    )
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
        |i| HttpResponse::Ok().json::<dto::InvoiceRow>(i.into()),
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
        |_| HttpResponse::Ok().body("{\"success\":true}"),
    )
    .await
}

#[post("/data-update/contact")]
pub async fn update_contact(
    contact: web::Json<Contact>,
    ctx: web::Data<RequestContext>,
) -> impl Responder {
    debug!("Updating contact: {:?}", contact);

    with_ok(ctx.dao.update_contact(&contact.into_inner().into()), |_| {
        HttpResponse::Ok().body("{\"success\":true}")
    })
    .await
}

#[post("/data-update/invoice")]
pub async fn update_invoice(
    invoice: web::Json<Invoice>,
    ctx: web::Data<RequestContext>,
) -> impl Responder {
    debug!("Deleting invoice: {:?}", invoice);

    with_ok(ctx.dao.update_invoice(&invoice.into_inner().into()), |_| {
        HttpResponse::Ok().body("{\"success\":true}")
    })
    .await
}

#[post("/data-update/invoice-row")]
pub async fn update_invoice_row(
    row: web::Json<InvoiceRow>,
    ctx: web::Data<RequestContext>,
) -> impl Responder {
    debug!("Deleting invoice row: {:?}", row);

    with_ok(ctx.dao.update_invoice_row(&row.into_inner().into()), |_| {
        HttpResponse::Ok().body("{\"success\":true}")
    })
    .await
}

#[post("/data-delete/entrepreneur/{id}")]
pub async fn delete_entrepreneur(
    id: web::Path<u32>,
    ctx: web::Data<RequestContext>,
) -> impl Responder {
    debug!("Deleting entrepreneur ID {}", id);

    with_ok(ctx.dao.delete_entrepreneur(id.into_inner()), |_| {
        HttpResponse::Ok().body("{\"success\":true}")
    })
    .await
}

#[post("/data-delete/contact/{id}")]
pub async fn delete_contact(id: web::Path<u32>, ctx: web::Data<RequestContext>) -> impl Responder {
    debug!("Deleting contact ID {}", id);

    with_ok(ctx.dao.delete_contact(id.into_inner()), |_| {
        HttpResponse::Ok().body("{\"success\":true}")
    })
    .await
}

#[post("/data-delete/invoice/{id}")]
pub async fn delete_invoice(id: web::Path<u32>, ctx: web::Data<RequestContext>) -> impl Responder {
    debug!("Updating invoice ID {}", id);

    with_ok(ctx.dao.delete_invoice(id.into_inner()), |_| {
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

    with_ok(ctx.dao.delete_invoice_row(id.into_inner()), |_| {
        HttpResponse::Ok().body("{\"success\":true}")
    })
    .await
}

async fn with_ok<A, F>(req: impl Future<Output = DaoResult<A>>, f: F) -> HttpResponse
where
    F: FnOnce(A) -> HttpResponse,
{
    match req.await {
        Ok(a) => f(a),
        Err(e) => {
            warn!("Error while querying DB: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

async fn with_found<A, F>(req: impl Future<Output = DaoResult<Option<A>>>, f: F) -> HttpResponse
where
    F: FnOnce(A) -> HttpResponse,
{
    with_ok(req, |r| match r {
        Some(a) => f(a),
        None => HttpResponse::NotFound().finish(),
    })
    .await
}
