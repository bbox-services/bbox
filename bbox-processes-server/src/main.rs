mod config;
mod dagster;
mod endpoints;
mod error;
mod models;

use actix_web::{middleware, App, HttpServer};
use bbox_common::api::{OgcApiInventory, OpenApiDoc};

#[actix_web::main]
pub async fn webserver() -> std::io::Result<()> {
    endpoints::init_service(&mut OgcApiInventory::new(), &mut OpenApiDoc::new());

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .configure(endpoints::register)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

fn main() {
    bbox_common::logger::init();
    webserver().unwrap();
}
