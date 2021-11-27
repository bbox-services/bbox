mod endpoints;
mod qgis_plugins;

use actix_web::{middleware, App, HttpServer};

#[actix_web::main]
pub async fn webserver() -> std::io::Result<()> {
    let plugins_index = endpoints::init_service();

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .configure(|mut cfg| endpoints::register(&mut cfg, &plugins_index))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

fn main() {
    bbox_common::logger::init();
    webserver().unwrap();
}
