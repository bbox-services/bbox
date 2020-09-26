use crate::fcgi_process::*;
use actix_web::{get, middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use log::{debug, error};
use std::io::{BufRead, Cursor, Read};

#[get("/")]
async fn index() -> String {
    "Ok".to_string()
}

#[get("/qgis/{project}")]
async fn qgis(
    fcgi: web::Data<FcgiClientHandler>,
    project: web::Path<String>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let mut fcgi_client = fcgi.fcgi_client()?;
    let query = format!("map=test/{}.qgs&{}", project, req.query_string());
    let params = fastcgi_client::Params::new()
        .set_request_method("GET")
        .set_query_string(&query);
    let output = fcgi_client
        .do_request(&params, &mut std::io::empty())
        .unwrap();
    let fcgiout = output.get_stdout().unwrap();

    let mut response = HttpResponse::Ok();

    let mut cursor = Cursor::new(fcgiout);
    let mut line = String::new();
    while let Ok(bytes) = cursor.read_line(&mut line) {
        if bytes <= 1 {
            break;
        }
        // Truncate newline
        if bytes > 0 {
            line.truncate(line.len() - 1);
        }
        let parts: Vec<&str> = line.split(": ").collect();
        if parts.len() != 2 {
            error!("{:?}", "Invalid FCGI-Header received");
            break;
        }
        let (key, value) = (parts[0], parts[1]);
        match key {
            "Content-Type" => {
                response.header(key, value);
            }
            _ => debug!("Ignoring FCGI-Header: {}", line),
        }
        line.truncate(0);
    }
    let mut body = Vec::new(); // TODO: return body without copy
    let _bytes = cursor.read_to_end(&mut body);
    Ok(response.body(body))
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
