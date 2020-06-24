use std::future::Future;

use actix_web::body::BodyStream;
use actix_web::{get, web, HttpResponse, Responder};
use log::{debug, warn};

use crate::dao::DaoResult;
use crate::RequestContext;

mod dto;

#[get("/download")]
pub async fn get(ctx: web::Data<RequestContext>) -> impl Responder {
    with_found(ctx.dao.load_contact(), |contact| {
        let pdf_stream = ctx.pdf_manager.create(contact.name);
        HttpResponse::Ok().body(BodyStream::new(pdf_stream))
    })
    .await
}

// TODO security: login and https://docs.rs/csrf/0.4.0/csrf/

#[get("/data/invoice/{id}")]
pub async fn ajax_action(
    invoice_id: web::Path<u32>,
    ctx: web::Data<RequestContext>,
) -> impl Responder {
    debug!("Getting invoice data, ID {}", invoice_id);

    with_found(ctx.dao.get_invoice(invoice_id.into_inner()), |i| {
        HttpResponse::Ok().json::<dto::Invoice>(i.into())
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
