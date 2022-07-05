mod endpoints;
mod engine;

use actix_web::{middleware, web, App, HttpServer};
use engine::Router;

#[actix_web::main]
pub async fn webserver() -> std::io::Result<()> {
    let router = Router::from_gpkg("../data/railway-test.gpkg", "flows", "geom")
        .await
        .unwrap();

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .app_data(web::Data::new(router.clone()))
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
