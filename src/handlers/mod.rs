use std::future::Future;
use std::pin::Pin;

use actix_web::body::BodyStream;
use actix_web::dev::{Payload, PayloadStream};
use actix_web::http::HeaderValue;
use actix_web::{get, post, web, FromRequest, HttpRequest, HttpResponse, Responder};
use err_context::AnyError;
use futures::future::{err, ok};
use futures::FutureExt;
use log::{debug, trace, warn};
use serde::Deserialize;

use crate::dao::DaoResult;
pub use crate::handlers::dto::LoginSession;
use crate::handlers::dto::{
    Contact, ContactsListParams, Entrepreneur, Invoice, InvoiceRow, InvoiceWithRows, InvoicesListParams, LoginSessionCreated, NewContact,
    NewEntrepreneur, NewInvoice, NewInvoiceRow,
};
use crate::logic;
use crate::logic::auth::Auth;
use crate::RequestContext;

pub mod dto;

#[post("/download/{id}")]
pub async fn download_invoice(id: web::Path<u32>, session: LoginSession, ctx: web::Data<RequestContext>) -> impl Responder {
    if !(session.is_valid_for_invoice(&ctx.dao, *id).await) {
        debug!("Session {:?} is forbidden to access invoice id {}", session, *id);
        return HttpResponse::Forbidden().body("Invalid resource");
    }

    match logic::download_invoice(&ctx.dao, &ctx.pdf_manager, *id).await {
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
pub async fn get_entrepreneur(entrepreneur_id: web::Path<u32>, session: LoginSession, ctx: web::Data<RequestContext>) -> impl Responder {
    debug!("Getting entrepreneur data, ID {}", entrepreneur_id);

    if !(session.is_valid_for_entrepreneur(&ctx.dao, *entrepreneur_id).await) {
        debug!("Session {:?} is forbidden to access entrepreneur id {}", session, *entrepreneur_id);
        return HttpResponse::Forbidden().body("Invalid resource");
    }

    with_found(ctx.dao.get_entrepreneur(*entrepreneur_id), |i| async {
        HttpResponse::Ok().json::<dto::Entrepreneur>(i.into())
    })
    .await
}

#[post("/data-get/invoice/{id}")]
pub async fn get_invoice(id: web::Path<u32>, session: LoginSession, ctx: web::Data<RequestContext>) -> impl Responder {
    debug!("Getting invoice data, ID {}", *id);

    if !(session.is_valid_for_invoice(&ctx.dao, *id).await) {
        debug!("Session {:?} is forbidden to access invoice id {}", session, *id);
        return HttpResponse::Forbidden().body("Invalid resource");
    }

    with_found(ctx.dao.get_invoice(*id), |i| async {
        HttpResponse::Ok().json::<dto::InvoiceWithAllInfo>(i.into())
    })
    .await
}

#[post("/data-get/invoice-with-rows/{id}")]
pub async fn get_invoice_with_rows(id: web::Path<u32>, session: LoginSession, ctx: web::Data<RequestContext>) -> impl Responder {
    debug!("Getting invoice data incl. rows, ID {}", *id);

    if !(session.is_valid_for_invoice(&ctx.dao, *id).await) {
        debug!("Session {:?} is forbidden to access invoice id {}", session, *id);
        return HttpResponse::Forbidden().body("Invalid resource");
    }

    with_found(ctx.dao.get_invoice_with_rows(*id), |(invoice, rows)| async {
        let iwr = InvoiceWithRows {
            invoice: invoice.into(),
            rows: rows.into_iter().map(|r| r.into()).collect(),
        };

        HttpResponse::Ok().json(iwr)
    })
    .await
}

#[post("/data-get/contact/{id}")]
pub async fn get_contact(id: web::Path<u32>, session: LoginSession, ctx: web::Data<RequestContext>) -> impl Responder {
    debug!("Getting contact data, ID {}", *id);

    if !(session.is_valid_for_contact(&ctx.dao, *id).await) {
        debug!("Session {:?} is forbidden to access contact id {}", session, *id);
        return HttpResponse::Forbidden().body("Invalid resource");
    }

    with_found(ctx.dao.get_contact(*id), |i| async {
        HttpResponse::Ok().json::<dto::Contact>(i.into())
    })
    .await
}

#[post("/data-get/entrepreneurs")]
pub async fn list_entrepreneurs(session: LoginSession, ctx: web::Data<RequestContext>) -> impl Responder {
    debug!("Getting entrepreneurs list for account ID {}", session.account_id);

    // no access rights check

    with_ok(ctx.dao.get_entrepreneurs(session.account_id), |rows| async {
        HttpResponse::Ok().json::<Vec<dto::Entrepreneur>>(rows.into_iter().map(|r| r.into()).collect())
    })
    .await
}

#[post("/data-get/contacts/{id}")]
pub async fn list_contacts(
    entrepreneur_id: web::Path<u32>,
    session: LoginSession,
    params: Option<web::Json<ContactsListParams>>,
    ctx: web::Data<RequestContext>,
) -> impl Responder {
    debug!("Getting contacts list for entrepreneur ID {}", entrepreneur_id);

    if !(session.is_valid_for_entrepreneur(&ctx.dao, *entrepreneur_id).await) {
        debug!("Session {:?} is forbidden to access entrepreneur id {}", session, *entrepreneur_id);
        return HttpResponse::Forbidden().body("Invalid resource");
    }

    let limit = params.as_ref().and_then(|p| p.count);
    let last_months = params.as_ref().and_then(|p| p.last_months);

    with_ok(ctx.dao.get_contacts(*entrepreneur_id, limit, last_months), |rows| async {
        HttpResponse::Ok().json::<Vec<dto::Contact>>(rows.into_iter().map(|r| r.into()).collect())
    })
    .await
}

#[post("/data-get/invoices/{id}")]
pub async fn list_invoices(
    entrepreneur_id: web::Path<u32>,
    session: LoginSession,
    params: Option<web::Json<InvoicesListParams>>,
    ctx: web::Data<RequestContext>,
) -> impl Responder {
    debug!("Getting invoices list for entrepreneur ID {}", entrepreneur_id);

    if !(session.is_valid_for_entrepreneur(&ctx.dao, *entrepreneur_id).await) {
        debug!("Session {:?} is forbidden to access entrepreneur id {}", session, *entrepreneur_id);
        return HttpResponse::Forbidden().body("Invalid resource");
    }

    let limit = params.and_then(|p| p.last);

    with_ok(ctx.dao.get_invoices(*entrepreneur_id, limit), |rows| async {
        HttpResponse::Ok().json::<Vec<dto::InvoiceWithAllInfo>>(rows.into_iter().map(|r| r.into()).collect())
    })
    .await
}

#[post("/data-get/invoice-rows/{id}")]
pub async fn list_invoice_rows(id: web::Path<u32>, session: LoginSession, ctx: web::Data<RequestContext>) -> impl Responder {
    debug!("Getting invoice rows data, ID {}", *id);

    if !(session.is_valid_for_invoice(&ctx.dao, *id).await) {
        debug!("Session {:?} is forbidden to access invoice id {}", session, *id);
        return HttpResponse::Forbidden().body("Invalid resource");
    }

    with_ok(ctx.dao.get_invoice_rows(*id), |rows| async {
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

    // TODO this has to be fixed
    with_ok(
        ctx.dao
            .insert_entrepreneur(&entrepreneur.code, &entrepreneur.name, &entrepreneur.address),
        |i| async { HttpResponse::Ok().json::<dto::Entrepreneur>(i.into()) },
    )
    .await
}

#[post("/data-insert/contact")]
pub async fn insert_contact(contact: web::Json<NewContact>, session: LoginSession, ctx: web::Data<RequestContext>) -> impl Responder {
    debug!("Inserting new contact: {:?}", contact);

    if !(session.is_valid_for_entrepreneur(&ctx.dao, contact.entrepreneur_id).await) {
        debug!(
            "Session {:?} is forbidden to insert contact for entrepreneur id {}",
            session, contact.entrepreneur_id
        );
        return HttpResponse::Forbidden().body("Invalid resource");
    }

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
pub async fn insert_invoice(invoice: web::Json<NewInvoice>, session: LoginSession, ctx: web::Data<RequestContext>) -> impl Responder {
    debug!("Inserting new invoice: {:?}", invoice);

    if !(session.is_valid_for_entrepreneur(&ctx.dao, invoice.entrepreneur_id).await
        && session.is_valid_for_contact(&ctx.dao, invoice.contact_id).await)
    {
        debug!(
            "Session {:?} is forbidden to insert new invoice for entrepreneur id {}",
            session, invoice.entrepreneur_id
        );
        return HttpResponse::Forbidden().body("Invalid resource");
    }

    with_ok(logic::insert_invoice(&ctx.dao, &invoice), |i| async {
        HttpResponse::Ok().json::<dto::InvoiceWithAllInfo>(i.into())
    })
    .await
}

#[post("/data-insert/invoice-row")]
pub async fn insert_invoice_row(row: web::Json<NewInvoiceRow>, session: LoginSession, ctx: web::Data<RequestContext>) -> impl Responder {
    debug!("Inserting new invoice row: {:?}", row);

    if !(session.is_valid_for_invoice(&ctx.dao, row.invoice_id).await) {
        debug!("Session {:?} is forbidden to access invoice id {}", session, row.invoice_id);
        return HttpResponse::Forbidden().body("Invalid resource");
    }

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
    session: LoginSession,
    ctx: web::Data<RequestContext>,
) -> impl Responder {
    debug!("Updating entrepreneur: {:?}", entrepreneur);

    if !(session.is_valid_for_entrepreneur(&ctx.dao, entrepreneur.id as u32).await) {
        debug!("Session {:?} is forbidden to access entrepreneur id {}", session, entrepreneur.id);
        return HttpResponse::Forbidden().body("Invalid resource");
    }

    with_ok(ctx.dao.update_entrepreneur(&entrepreneur.into_inner().into()), |_| async {
        HttpResponse::Ok().body("{\"success\":true}")
    })
    .await
}

#[post("/data-update/contact")]
pub async fn update_contact(contact: web::Json<Contact>, session: LoginSession, ctx: web::Data<RequestContext>) -> impl Responder {
    debug!("Updating contact: {:?}", contact);

    if !(session.is_valid_for_contact(&ctx.dao, contact.id as u32).await) {
        debug!("Session {:?} is forbidden to access contact id {}", session, contact.id);
        return HttpResponse::Forbidden().body("Invalid resource");
    }

    with_ok(ctx.dao.update_contact(&contact.into_inner().into()), |_| async {
        HttpResponse::Ok().body("{\"success\":true}")
    })
    .await
}

#[post("/data-update/invoice")]
pub async fn update_invoice(invoice: web::Json<Invoice>, session: LoginSession, ctx: web::Data<RequestContext>) -> impl Responder {
    debug!("Updating invoice: {:?}", invoice);

    if !(session.is_valid_for_invoice(&ctx.dao, invoice.id as u32).await) {
        debug!("Session {:?} is forbidden to access invoice id {}", session, invoice.id);
        return HttpResponse::Forbidden().body("Invalid resource");
    }

    with_ok(ctx.dao.update_invoice(&invoice.into_inner().into()), |_| async {
        HttpResponse::Ok().body("{\"success\":true}")
    })
    .await
}

#[post("/data-update/invoice-row")]
pub async fn update_invoice_row(row: web::Json<InvoiceRow>, session: LoginSession, ctx: web::Data<RequestContext>) -> impl Responder {
    debug!("Updating invoice row: {:?}", row);

    if !(session.is_valid_for_invoice(&ctx.dao, row.invoice_id as u32).await) {
        debug!("Session {:?} is forbidden to access invoice id {}", session, row.invoice_id);
        return HttpResponse::Forbidden().body("Invalid resource");
    }

    with_ok(ctx.dao.update_invoice_row(&row.into_inner().into()), |_| async {
        HttpResponse::Ok().body("{\"success\":true}")
    })
    .await
}

#[post("/data-delete/entrepreneur/{id}")]
pub async fn delete_entrepreneur(id: web::Path<u32>, session: LoginSession, ctx: web::Data<RequestContext>) -> impl Responder {
    debug!("Deleting entrepreneur ID {}", id);

    if !(session.is_valid_for_entrepreneur(&ctx.dao, *id).await) {
        debug!("Session {:?} is forbidden to access entrepreneur id {}", session, *id);
        return HttpResponse::Forbidden().body("Invalid resource");
    }

    with_ok(ctx.dao.delete_entrepreneur(*id), |_| async {
        HttpResponse::Ok().body("{\"success\":true}")
    })
    .await
}

#[post("/data-delete/contact/{id}")]
pub async fn delete_contact(id: web::Path<u32>, session: LoginSession, ctx: web::Data<RequestContext>) -> impl Responder {
    debug!("Deleting contact ID {}", id);

    if !(session.is_valid_for_contact(&ctx.dao, *id).await) {
        debug!("Session {:?} is forbidden to access contact id {}", session, *id);
        return HttpResponse::Forbidden().body("Invalid resource");
    }

    with_ok(ctx.dao.delete_contact(*id), |_| async {
        HttpResponse::Ok().body("{\"success\":true}")
    })
    .await
}

#[post("/data-delete/invoice/{id}")]
pub async fn delete_invoice(id: web::Path<u32>, session: LoginSession, ctx: web::Data<RequestContext>) -> impl Responder {
    debug!("Deleting invoice ID {}", id);

    if !(session.is_valid_for_invoice(&ctx.dao, *id).await) {
        debug!("Session {:?} is forbidden to access invoice id {}", session, *id);
        return HttpResponse::Forbidden().body("Invalid resource");
    }

    with_ok(ctx.dao.delete_invoice(*id), |_| async {
        HttpResponse::Ok().body("{\"success\":true}")
    })
    .await
}

#[post("/data-delete/invoice-row/{id}")]
pub async fn delete_invoice_row(id: web::Path<u32>, session: LoginSession, ctx: web::Data<RequestContext>) -> impl Responder {
    debug!("Deleting invoice row ID {}", id);

    if !(session.is_valid_for_invoice_row(&ctx.dao, *id).await) {
        debug!("Session {:?} is forbidden to access invoice id {}", session, *id);
        return HttpResponse::Forbidden().body("Invalid resource");
    }

    with_ok(ctx.dao.delete_invoice_row(*id), |_| async {
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

    fn from_request(req: &HttpRequest, payload: &mut Payload<PayloadStream>) -> Self::Future {
        let config = req.app_data::<LoginSessionExtractorConfig>().expect("Could not extract config");
        let ctx = config.ctx.as_ref().expect("Request config not available").clone();
        use futures::StreamExt;

        if req.query_string().contains("auth=1") {
            let mut payload = payload.take();

            async move {
                debug!("Trying authentication via body");
                let mut bytes = web::BytesMut::new();
                while let Some(item) = payload.next().await {
                    let item = item?;
                    trace!("Chunk: {:?}", &item);
                    bytes.extend_from_slice(&item);
                }
                let bytes = bytes.to_vec();
                let auth = String::from_utf8(bytes).map_err(|e| e.utf8_error())?;
                debug!("Provided session: {}", auth);

                match LoginSession::try_parse_from_body(&auth) {
                    Ok(session) => LoginSession::try_authenticate(ctx, session),
                    Err(e) => {
                        debug!("Could not authenticate via body token: {:?}", e);
                        Box::pin(err(actix_web::error::ErrorUnauthorized("Invalid auth provided")))
                    }
                }
                .await
            }
            .boxed_local()
        } else if let Some(header) = req.headers().get("X-Faktury-Auth") {
            match LoginSession::try_parse_from_header(header) {
                Ok(session) => LoginSession::try_authenticate(ctx, session),
                Err(e) => {
                    debug!("Could not authenticate via header: {:?}", e);
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
    fn try_parse_from_header(header: &HeaderValue) -> Result<LoginSession, AnyError> {
        let str = header.to_str()?;
        LoginSession::try_parse(str)
    }

    fn try_parse_from_body(value: &str) -> Result<LoginSession, AnyError> {
        use percent_encoding::percent_decode_str;

        let value = value
            .strip_prefix("auth=")
            .ok_or_else(|| AnyError::from("Could not handle provided auth token"))?;

        // url decode
        let value = percent_decode_str(value).decode_utf8()?;

        LoginSession::try_parse(value.as_ref())
    }

    fn try_parse(value: &str) -> Result<LoginSession, AnyError> {
        let decoded = base64::decode(value)?;
        let result = serde_json::from_slice(decoded.as_slice())?;
        Ok(result)
    }

    fn try_authenticate(
        ctx: RequestContext,
        session: LoginSession,
    ) -> Pin<Box<dyn Future<Output = Result<LoginSession, actix_web::Error>>>> {
        async move {
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
        .boxed_local()
    }
}
