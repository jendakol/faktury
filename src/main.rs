// TODO WTH is this needed
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use std::convert::TryFrom;
use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::str::FromStr;

use actix_cors::Cors;
use actix_files::NamedFile;
use actix_web::{middleware, web, App, HttpRequest, HttpServer, Result as ActixResult};
use log::{debug, info, trace};

use crate::config::{AccountsConfig, AppConfig};
use crate::dao::Dao;
use crate::logic::pdf::PdfManager;

mod config;
mod dao;
mod handlers;
mod logic;

#[derive(Clone)]
pub struct RequestContext {
    dao: Dao,
    pdf_manager: PdfManager,
    accounts_config: AccountsConfig,
}

async fn web_ui(req: HttpRequest) -> ActixResult<NamedFile> {
    let path: PathBuf = req.match_info().query("filename").parse()?;

    trace!("Requesting {:?}", path);

    if path.starts_with("imgs") || path.starts_with("css") || path.starts_with("js") {
        let mut resource_path = PathBuf::from("static");
        resource_path.push(path);

        trace!("Returning {:?} resource", resource_path);
        Ok(NamedFile::open(resource_path)?)
    } else {
        trace!("Returning index file");
        Ok(NamedFile::open("static/index.html")?)
    }
}

#[actix_rt::main]
async fn main() {
    env_logger::init();

    let config_file = env::var("CONFIG_FILE").unwrap_or_else(|_| String::from("/config.toml"));

    debug!("Using config file {}", config_file);

    let config = AppConfig::load(&config_file).expect("Could not load configuration file!"); // let it fail
    let dao = Dao::try_from(config.database.clone()).expect("Could not initialize DB connection!"); // let it fail
    let pdf_manager = PdfManager::new().expect("Could not initialize PDF manager!"); // let it fail
    let addr = SocketAddr::from_str(&config.http.listen).expect("Could not parse listen address!"); // let it fail

    info!("Starting server on {}", addr);

    // TODO CORS headers

    HttpServer::new(move || {
        let request_context = RequestContext {
            dao: dao.clone(),
            pdf_manager: pdf_manager.clone(),
            accounts_config: config.accounts.clone(),
        };

        let cors = config
            .http
            .cors_origins
            .iter()
            .fold(Cors::default(), |c, origin| c.allowed_origin(origin))
            .allow_any_method()
            .allow_any_header()
            .supports_credentials();

        App::new()
            .app_data(web::Data::new(request_context))
            .wrap(cors)
            .wrap(middleware::Compress::default())
            .service(handlers::download_invoice)
            .service(handlers::account_login)
            .service(handlers::account_logout)
            .service(handlers::get_entrepreneur)
            .service(handlers::get_contact)
            .service(handlers::get_invoice)
            .service(handlers::get_invoice_with_rows)
            .service(handlers::list_entrepreneurs)
            .service(handlers::list_contacts)
            .service(handlers::list_invoices)
            .service(handlers::list_invoice_rows)
            .service(handlers::insert_entrepreneur)
            .service(handlers::insert_contact)
            .service(handlers::insert_invoice)
            .service(handlers::copy_invoice)
            .service(handlers::insert_invoice_row)
            .service(handlers::update_entrepreneur)
            .service(handlers::update_contact)
            .service(handlers::update_invoice)
            .service(handlers::update_invoice_row)
            .service(handlers::delete_entrepreneur)
            .service(handlers::delete_contact)
            .service(handlers::delete_invoice)
            .service(handlers::delete_invoice_row)
            .service(handlers::get_yearly_stats)
            .service(handlers::login_salt)
            .service(handlers::status)
            .route("/{filename:.*}", web::get().to(web_ui))
    })
    .bind(addr)
    .unwrap() // let it fail
    .run()
    .await
    .unwrap();
}
