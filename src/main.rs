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
use log::{info, trace};

use crate::config::AppConfig;
use crate::dao::Dao;
use crate::pdf::PdfManager;

mod config;
mod dao;
mod handlers;
mod pdf;

pub struct RequestContext {
    dao: Dao,
    pdf_manager: PdfManager,
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

    let config = AppConfig::load(&config_file).unwrap(); // let it fail
    let dao = Dao::try_from(config.database).unwrap(); // let it fail
    let pdf_manager = PdfManager::new().unwrap(); // let it fail
    let addr = SocketAddr::from_str(&config.http.listen).unwrap(); // let it fail

    info!("Starting server on {}", addr);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(RequestContext {
                dao: dao.clone(),
                pdf_manager: pdf_manager.clone(),
            }))
            .wrap(Cors::new().supports_credentials().finish()) // TODO limit
            .wrap(middleware::Compress::default())
            .service(handlers::get_entrepreneur)
            .service(handlers::get_contact)
            .service(handlers::get_invoice)
            .service(handlers::get_invoice_with_rows)
            .service(handlers::list_contacts)
            .service(handlers::list_invoices)
            .service(handlers::list_invoice_rows)
            .service(handlers::insert_entrepreneur)
            .service(handlers::insert_contact)
            .service(handlers::insert_invoice)
            .service(handlers::insert_invoice_row)
            .service(handlers::update_entrepreneur)
            .service(handlers::update_contact)
            .service(handlers::update_invoice)
            .service(handlers::update_invoice_row)
            .service(handlers::delete_entrepreneur)
            .service(handlers::delete_contact)
            .service(handlers::delete_invoice)
            .service(handlers::delete_invoice_row)
            .route("/{filename:.*}", web::get().to(web_ui))
    })
    .bind(addr)
    .unwrap() // let it fail
    .run()
    .await
    .unwrap();
}
