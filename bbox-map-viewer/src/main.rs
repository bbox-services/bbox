mod qwc2_config;
mod static_files;
mod webserver;

use actix_web::{middleware, App, HttpServer};

#[actix_web::main]
pub async fn webserver() -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .configure(webserver::register_endpoints)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}

fn main() {
    bbox_common::logger::init();
    webserver().unwrap();
}
