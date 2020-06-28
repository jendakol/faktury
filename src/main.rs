// TODO WTH is this needed
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use std::convert::TryFrom;
use std::env;
use std::net::SocketAddr;
use std::str::FromStr;

use actix_web::{middleware, web, App, HttpResponse, HttpServer};
use log::info;

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

#[actix_rt::main]
async fn main() {
    env_logger::init();

    let config_file = env::var("CONFIG_FILE").unwrap_or_else(|_| String::from("/config.toml"));

    let config = AppConfig::load(&config_file).unwrap(); // let it fail
    let dao = Dao::try_from(config.database).unwrap(); // let it fail
    let pdf_manager = PdfManager::new().unwrap(); // let it fail
    let addr = SocketAddr::from_str(&config.http.listen).unwrap(); // let it fail

    // insert test data; don't fail, if they exist
    let ent_id = dao
        .insert_entrepreneur("123456789", "Pokusný", "Prdelákov")
        .await
        .map(|e| e.id)
        .unwrap_or(1);

    let _ = dao
        .insert_contact(ent_id as u32, "123456789", "Pokusný", "Prdelákov")
        .await;

    info!("Starting server on {}", addr);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(RequestContext {
                dao: dao.clone(),
                pdf_manager: pdf_manager.clone(),
            }))
            .wrap(middleware::Compress::default())
            .service(handlers::get_entrepreneur)
            .service(handlers::get_contact)
            .service(handlers::get_invoice)
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
            .default_service(web::route().to(HttpResponse::NotFound))
    })
    .bind(addr)
    .unwrap() // let it fail
    .run()
    .await
    .unwrap();
}
