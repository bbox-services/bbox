mod config;
mod dispatcher;
mod endpoints;
mod fcgi_process;
mod init;
mod inventory;
mod wms_capabilities;
mod wms_fcgi_backend;

use actix_web::{middleware, App, HttpServer};
use bbox_common::api::{OgcApiInventory, OpenApiDoc};

#[actix_web::main]
pub async fn webserver() -> std::io::Result<()> {
    let (fcgi_clients, inventory) =
        init::init_service(&mut OgcApiInventory::new(), &mut OpenApiDoc::new(), None).await;

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .configure(|mut cfg| endpoints::register(&mut cfg, &fcgi_clients, &inventory))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

fn main() {
    bbox_common::logger::init();
    webserver().unwrap();
}
