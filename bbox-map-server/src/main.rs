mod dispatcher;
mod fcgi_process;
mod file_search;
pub mod inventory;
mod webserver;
mod wms_capabilities;
mod wms_fcgi_backend;

use actix_web::{middleware, App, HttpServer};

#[actix_web::main]
pub async fn webserver() -> std::io::Result<()> {
    let (fcgi_clients, inventory) = wms_fcgi_backend::init_service().await;

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .configure(|mut cfg| webserver::register_endpoints(&mut cfg, &fcgi_clients, &inventory))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}

fn main() {
    bbox_common::logger::init();
    webserver().unwrap();
}
