use actix_web::body::BodyStream;
use actix_web::{get, web, HttpResponse, Responder};

use crate::RequestContext;

#[get("/download")]
pub async fn get(rctx: web::Data<RequestContext>) -> impl Responder {
    let contact = rctx.dao.load_contact().await;
    let pdf_stream = rctx.pdf_manager.create(contact.name);
    HttpResponse::Ok().body(BodyStream::new(pdf_stream)).await
}
