use std::future::Future;
use std::ops::Deref;
use std::pin::Pin;

use actix_web::body::BodyStream;
use actix_web::dev::{Payload, PayloadStream};
use actix_web::http::HeaderValue;
use actix_web::{get, post, web, FromRequest, HttpRequest, HttpResponse, Responder};
use err_context::AnyError;
use futures::future::{err, ok};
use futures::FutureExt;
use log::{debug, warn};
use serde::Deserialize;

use crate::dao::DaoResult;
pub use crate::handlers::dto::LoginSession;
use crate::handlers::dto::{
    Contact, Entrepreneur, Invoice, InvoiceRow, InvoiceWithRows, LoginSessionCreated, NewContact, NewEntrepreneur, NewInvoice,
    NewInvoiceRow,
};
use crate::logic;
use crate::logic::invoices as InvoicesLogic;
use crate::logic::settings::AccountSettings;
use crate::RequestContext;

mod dto;

#[get("/download/{id}")]
pub async fn download_invoice(id: web::Path<u32>, ctx: web::Data<RequestContext>) -> impl Responder {
    match logic::download_invoice(id.into_inner(), &ctx.dao, &ctx.pdf_manager).await {
        Ok((invoice, stream)) => {
            let hvalue = format!("attachment; filename=\"invoice_{}.pdf\"", invoice.code);

            HttpResponse::Ok()
                .set_header("Content-Type", "application/pdf")
                .set_header("Content-Disposition", hvalue)
                .body(BodyStream::new(stream))
        }
        Err(err) => {
            warn!("Error while downloading invoice PDF: {}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}

// TODO security: https://docs.rs/csrf/0.4.0/csrf/

#[derive(Deserialize)]
pub struct Login {
    username: String,
    password: String,
}

#[post("/account-login")]
pub async fn account_login(data: web::Json<Login>, ctx: web::Data<RequestContext>) -> impl Responder {
    debug!("Trying to login as {}", data.username);

    with_ok(ctx.dao.find_account(&data.username, &data.password), |account| async {
        match account {
            Some(account) => {
                debug!("Found account for {}", data.username);
                with_ok(ctx.dao.new_session(&account), |session| async {
                    let session = LoginSession::from(session); // DAO to DTO entity
                    debug!("Created new session for {}: {:?}", data.username, &session);
                    // TODO sign session
                    let session_encoded = base64::encode(serde_json::to_string(&session).expect("Could not serialize session"));
                    HttpResponse::Ok().json(LoginSessionCreated {
                        encoded_value: session_encoded,
                        ttl: ctx.accounts_config.login_ttl.num_milliseconds() as u64,
                    })
                })
                .await
            }
            None => {
                debug!("Could not login as {}", data.username);
                HttpResponse::Unauthorized().finish()
            }
        }
    })
    .await
}

#[post("/account-logout")]
pub async fn account_logout(session: LoginSession, ctx: web::Data<RequestContext>) -> impl Responder {
    debug!("Logging out");

    debug!("Revoking session {:?}", session.id);

    with_ok(ctx.dao.revoke_session(session.id), |_| async {
        // id_store.forget();
        HttpResponse::Ok().body("{\"success\": true}")
    })
    .await
}

#[post("/data-get/entrepreneur/{id}")]
pub async fn get_entrepreneur(entrepreneur_id: web::Path<u32>, _session: LoginSession, ctx: web::Data<RequestContext>) -> impl Responder {
    debug!("Getting entrepreneur data, ID {}", entrepreneur_id);

    // TODO check account rights
    with_found(ctx.dao.get_entrepreneur(entrepreneur_id.into_inner()), |i| async {
        HttpResponse::Ok().json::<dto::Entrepreneur>(i.into())
    })
    .await
}

#[post("/data-get/invoice/{id}")]
pub async fn get_invoice(invoice_id: web::Path<u32>, _session: LoginSession, ctx: web::Data<RequestContext>) -> impl Responder {
    debug!("Getting invoice data, ID {}", invoice_id);

    // TODO check account rights
    with_found(ctx.dao.get_invoice(invoice_id.into_inner()), |i| async {
        HttpResponse::Ok().json::<dto::InvoiceWithAllInfo>(i.into())
    })
    .await
}

#[post("/data-get/invoice-with-rows/{id}")]
pub async fn get_invoice_with_rows(invoice_id: web::Path<u32>, _session: LoginSession, ctx: web::Data<RequestContext>) -> impl Responder {
    debug!("Getting invoice data incl. rows, ID {}", invoice_id);

    // TODO check account rights
    with_found(ctx.dao.get_invoice_with_rows(invoice_id.into_inner()), |(invoice, rows)| async {
        let iwr = InvoiceWithRows {
            invoice: invoice.into(),
            rows: rows.into_iter().map(|r| r.into()).collect(),
        };

        HttpResponse::Ok().json(iwr)
    })
    .await
}

#[post("/data-get/contact/{id}")]
pub async fn get_contact(contact_id: web::Path<u32>, _session: LoginSession, ctx: web::Data<RequestContext>) -> impl Responder {
    debug!("Getting contact data, ID {}", contact_id);

    // TODO check account rights
    with_found(ctx.dao.get_contact(contact_id.into_inner()), |i| async {
        HttpResponse::Ok().json::<dto::Contact>(i.into())
    })
    .await
}

#[post("/data-get/contacts/{id}")]
pub async fn list_contacts(ent_id: web::Path<u32>, _session: LoginSession, ctx: web::Data<RequestContext>) -> impl Responder {
    debug!("Getting contacts list for entrepreneur ID {}", ent_id);

    // TODO check account rights
    with_ok(ctx.dao.get_contacts(ent_id.into_inner()), |rows| async {
        HttpResponse::Ok().json::<Vec<dto::Contact>>(rows.into_iter().map(|r| r.into()).collect())
    })
    .await
}

#[post("/data-get/invoices/{id}")]
pub async fn list_invoices(ent_id: web::Path<u32>, _session: LoginSession, ctx: web::Data<RequestContext>) -> impl Responder {
    debug!("Getting invoices list for entrepreneur ID {}", ent_id);

    // TODO check account rights
    with_ok(ctx.dao.get_invoices(ent_id.into_inner()), |rows| async {
        HttpResponse::Ok().json::<Vec<dto::InvoiceWithAllInfo>>(rows.into_iter().map(|r| r.into()).collect())
    })
    .await
}

#[post("/data-get/invoice-rows/{id}")]
pub async fn list_invoice_rows(invoice_id: web::Path<u32>, _session: LoginSession, ctx: web::Data<RequestContext>) -> impl Responder {
    debug!("Getting invoice rows data, ID {}", invoice_id);

    // TODO check account rights
    with_ok(ctx.dao.get_invoice_rows(invoice_id.into_inner()), |rows| async {
        HttpResponse::Ok().json::<Vec<dto::InvoiceRow>>(rows.into_iter().map(|r| r.into()).collect())
    })
    .await
}

#[post("/data-insert/entrepreneur")]
pub async fn insert_entrepreneur(
    entrepreneur: web::Json<NewEntrepreneur>,
    _session: LoginSession,
    ctx: web::Data<RequestContext>,
) -> impl Responder {
    debug!("Inserting new entrepreneur: {:?}", entrepreneur);

    // TODO check account rights
    with_ok(
        ctx.dao
            .insert_entrepreneur(&entrepreneur.code, &entrepreneur.name, &entrepreneur.address),
        |i| async { HttpResponse::Ok().json::<dto::Entrepreneur>(i.into()) },
    )
    .await
}

#[post("/data-insert/contact")]
pub async fn insert_contact(contact: web::Json<NewContact>, _session: LoginSession, ctx: web::Data<RequestContext>) -> impl Responder {
    debug!("Inserting new contact: {:?}", contact);

    // TODO check account rights
    with_ok(
        ctx.dao.insert_contact(
            contact.entrepreneur_id,
            &contact.code,
            &contact.name,
            &contact.address,
            &contact.vat,
        ),
        |i| async { HttpResponse::Ok().json::<dto::Contact>(i.into()) },
    )
    .await
}

#[post("/data-insert/invoice")]
pub async fn insert_invoice(invoice: web::Json<NewInvoice>, _session: LoginSession, ctx: web::Data<RequestContext>) -> impl Responder {
    use InvoicesLogic::next_code;

    debug!("Inserting new invoice: {:?}", invoice);

    // TODO check account rights
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
pub async fn insert_invoice_row(row: web::Json<NewInvoiceRow>, _session: LoginSession, ctx: web::Data<RequestContext>) -> impl Responder {
    debug!("Inserting new invoice row: {:?}", row);

    // TODO check account rights
    with_ok(
        ctx.dao
            .insert_invoice_row(row.invoice_id, &row.item_name, row.item_price, row.item_count),
        |i| async { HttpResponse::Ok().json::<dto::InvoiceRow>(i.into()) },
    )
    .await
}

#[post("/data-update/entrepreneur")]
pub async fn update_entrepreneur(
    entrepreneur: web::Json<Entrepreneur>,
    _session: LoginSession,
    ctx: web::Data<RequestContext>,
) -> impl Responder {
    debug!("Updating entrepreneur: {:?}", entrepreneur);

    // TODO check account rights
    with_ok(ctx.dao.update_entrepreneur(&entrepreneur.into_inner().into()), |_| async {
        HttpResponse::Ok().body("{\"success\":true}")
    })
    .await
}

#[post("/data-update/contact")]
pub async fn update_contact(contact: web::Json<Contact>, _session: LoginSession, ctx: web::Data<RequestContext>) -> impl Responder {
    debug!("Updating contact: {:?}", contact);

    // TODO check account rights
    with_ok(ctx.dao.update_contact(&contact.into_inner().into()), |_| async {
        HttpResponse::Ok().body("{\"success\":true}")
    })
    .await
}

#[post("/data-update/invoice")]
pub async fn update_invoice(invoice: web::Json<Invoice>, _session: LoginSession, ctx: web::Data<RequestContext>) -> impl Responder {
    debug!("Deleting invoice: {:?}", invoice);

    // TODO check account rights
    with_ok(ctx.dao.update_invoice(&invoice.into_inner().into()), |_| async {
        HttpResponse::Ok().body("{\"success\":true}")
    })
    .await
}

#[post("/data-update/invoice-row")]
pub async fn update_invoice_row(row: web::Json<InvoiceRow>, _session: LoginSession, ctx: web::Data<RequestContext>) -> impl Responder {
    debug!("Deleting invoice row: {:?}", row);

    // TODO check account rights
    with_ok(ctx.dao.update_invoice_row(&row.into_inner().into()), |_| async {
        HttpResponse::Ok().body("{\"success\":true}")
    })
    .await
}

#[post("/data-delete/entrepreneur/{id}")]
pub async fn delete_entrepreneur(id: web::Path<u32>, _session: LoginSession, ctx: web::Data<RequestContext>) -> impl Responder {
    debug!("Deleting entrepreneur ID {}", id);

    // TODO check account rights
    with_ok(ctx.dao.delete_entrepreneur(id.into_inner()), |_| async {
        HttpResponse::Ok().body("{\"success\":true}")
    })
    .await
}

#[post("/data-delete/contact/{id}")]
pub async fn delete_contact(id: web::Path<u32>, _session: LoginSession, ctx: web::Data<RequestContext>) -> impl Responder {
    debug!("Deleting contact ID {}", id);

    // TODO check account rights
    with_ok(ctx.dao.delete_contact(id.into_inner()), |_| async {
        HttpResponse::Ok().body("{\"success\":true}")
    })
    .await
}

#[post("/data-delete/invoice/{id}")]
pub async fn delete_invoice(id: web::Path<u32>, _session: LoginSession, ctx: web::Data<RequestContext>) -> impl Responder {
    debug!("Updating invoice ID {}", id);

    // TODO check account rights
    with_ok(ctx.dao.delete_invoice(id.into_inner()), |_| async {
        HttpResponse::Ok().body("{\"success\":true}")
    })
    .await
}

#[post("/data-delete/invoice-row/{id}")]
pub async fn delete_invoice_row(id: web::Path<u32>, _session: LoginSession, ctx: web::Data<RequestContext>) -> impl Responder {
    debug!("Updating invoice row ID {}", id);

    // TODO check account rights
    with_ok(ctx.dao.delete_invoice_row(id.into_inner()), |_| async {
        HttpResponse::Ok().body("{\"success\":true}")
    })
    .await
}

#[get("/status")]
pub async fn status() -> impl Responder {
    // TODO implement proper status
    HttpResponse::Ok().body("{\"status\":\"ok\"}")
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

impl FromRequest for LoginSession {
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn futures::Future<Output = Result<Self, Self::Error>>>>;
    type Config = LoginSessionExtractorConfig;

    fn from_request(req: &HttpRequest, _payload: &mut Payload<PayloadStream>) -> Self::Future {
        let config = req.app_data::<LoginSessionExtractorConfig>().expect("Could not extract config");
        let ctx = config.ctx.as_ref().expect("Request config not available").clone();

        if let Some(header) = req.headers().get("X-Faktury-Auth") {
            match LoginSession::extract_header_value(header) {
                Ok(session) => async move {
                    let found_session = ctx
                        .dao
                        .find_session(&session.id)
                        .await
                        .expect("Search for existing session has failed")
                        .map(Into::<LoginSession>::into);

                    match found_session {
                        Some(db_session) if db_session == session => {
                            debug!("Authenticated session: {:?}", session);
                            ok(session)
                        }
                        Some(db_session) => {
                            debug!("Session found but mismatch: {:?} vs {:?}", session, db_session);
                            err(actix_web::error::ErrorUnauthorized("Invalid auth provided"))
                        }
                        None => {
                            debug!("Could not find provided session {:?}", session);
                            err(actix_web::error::ErrorUnauthorized("Invalid auth provided"))
                        }
                    }
                    .await
                }
                .boxed_local(),
                Err(e) => {
                    debug!("Could not authenticate: {:?}", e);
                    Box::pin(err(actix_web::error::ErrorUnauthorized("Invalid auth provided")))
                }
            }
        } else {
            Box::pin(err(actix_web::error::ErrorUnauthorized("No auth provided")))
        }
    }
}

#[derive(Default, Clone)]
pub struct LoginSessionExtractorConfig {
    pub ctx: Option<RequestContext>,
}

impl LoginSession {
    fn extract_header_value(header: &HeaderValue) -> Result<LoginSession, AnyError> {
        let str = header.to_str()?;
        let decoded = base64::decode(str)?;
        let result = serde_json::from_slice(decoded.as_slice())?;
        Ok(result)
    }
}
