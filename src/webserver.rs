use crate::fcgi_process::*;
use actix_web::{get, web, App, Error, HttpResponse, HttpServer};
use deadpool::unmanaged::Pool;

#[get("/")]
async fn index() -> String {
    "Ok".to_string()
}

#[get("/qgis/{project}")]
async fn qgis(
    pool: web::Data<Pool<FcgiClient>>,
    project: web::Path<String>,
) -> Result<HttpResponse, Error> {
    let mut fcgi_client = pool.get().await;
    do_request(
        &mut fcgi_client,
        &format!(
            "map=test/{}.qgs&SERVICE=WMS&REQUEST=GetCapabilities",
            project
        ),
    )?;

    Ok(HttpResponse::Ok().json(42))
}

#[actix_web::main]
pub async fn webserver() -> std::io::Result<()> {
    let process = FcgiProcess::spawn("/usr/lib/cgi-bin/qgis_mapserv.fcgi").await?;
    process.wait_until_ready();
    let handler = process.handler();
    let pool = Pool::from(vec![
        handler.fcgi_client().unwrap(),
        handler.fcgi_client().unwrap(),
        handler.fcgi_client().unwrap(),
        handler.fcgi_client().unwrap(),
        handler.fcgi_client().unwrap(),
        handler.fcgi_client().unwrap(),
        handler.fcgi_client().unwrap(),
        handler.fcgi_client().unwrap(),
    ]);

    HttpServer::new(move || App::new().data(pool.clone()).service(index).service(qgis))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
