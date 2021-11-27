mod ogcapi;
mod openapi;
mod webserver;

use actix_web::{middleware, App, HttpServer};
use serde::Deserialize;

fn main() {
    bbox_common::logger::init();
    webserver().unwrap();
}

#[derive(Deserialize)]
struct WebserverConfig {
    server_addr: String,
}

#[actix_web::main]
pub async fn webserver() -> std::io::Result<()> {
    let config = bbox_common::config::app_config();
    let web_config: WebserverConfig = config
        .extract_inner("webserver")
        .expect("webserver config invalid");

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .configure(webserver::register_endpoints)
    })
    .bind(web_config.server_addr.clone())?
    .run()
    .await
}
