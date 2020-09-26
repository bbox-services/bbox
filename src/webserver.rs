use crate::fcgi_process::*;
use actix_web::{get, middleware, web, App, Error, HttpResponse, HttpServer};

#[get("/")]
async fn index() -> String {
    "Ok".to_string()
}

#[get("/qgis/{project}")]
async fn qgis(
    fcgi: web::Data<FcgiClientHandler>,
    project: web::Path<String>,
) -> Result<HttpResponse, Error> {
    let mut fcgi_client = fcgi.fcgi_client()?;
    let query = format!(
        "map=test/{}.qgs&SERVICE=WMS&REQUEST=GetCapabilities",
        project
    );
    let params = fastcgi_client::Params::new()
        .set_request_method("GET")
        .set_query_string(&query);
    let output = fcgi_client
        .do_request(&params, &mut std::io::empty())
        .unwrap();
    let fcgiout = output.get_stdout().unwrap();

    // use futures_lite::*
    // let mut reader = io::BufReader::new(fcgiout.take(99));
    // let mut lines = reader.lines();
    // let mut bytes = reader.bytes();
    // let mut s = stream::iter(fcgiout);

    Ok(HttpResponse::Ok().body(fcgiout))
}

#[actix_web::main]
pub async fn webserver() -> std::io::Result<()> {
    let process = FcgiProcess::spawn("/usr/lib/cgi-bin/qgis_mapserv.fcgi").await?;
    process.wait_until_ready();
    let handler = process.handler();

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Compress::default())
            .data(handler.clone())
            .service(index)
            .service(qgis)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
