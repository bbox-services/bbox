mod config;
mod datasource;
mod endpoints;
mod error;
mod filter_params;
mod inventory;

use actix_web::{middleware, App, HttpServer};
use bbox_common::api::{OgcApiInventory, OpenApiDoc};

#[actix_web::main]
pub async fn webserver() -> std::io::Result<()> {
    let inventory =
        endpoints::init_service(&mut OgcApiInventory::new(), &mut OpenApiDoc::new()).await;

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .configure(|mut cfg| endpoints::register(&mut cfg, &inventory))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

fn main() {
    bbox_common::logger::init();
    webserver().unwrap();
}
