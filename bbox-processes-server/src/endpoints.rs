use crate::dagster;
use crate::models::*;
use actix_web::{web, HttpRequest, HttpResponse};
use bbox_common::api::{ExtendApiDoc, OgcApiInventory};
use bbox_common::ogcapi::ApiLink;
use utoipa::OpenApi;

/// retrieve the list of available processes
///
/// The list of processes contains a summary of each process the OGC API -
/// Processes offers, including the link to a more detailed description
/// of the process.
#[utoipa::path(
    get,
    path = "/processes",
    operation_id = "ProcessList",
    tag = "processes",
    responses(
        (status = 200, body = ProcessList),
    ),
)]
async fn processes(_req: HttpRequest) -> HttpResponse {
    let jobs = dagster::query_jobs().await;
    let processes = jobs
        .iter()
        .map(|job| {
            let mut process = ProcessSummary::new(job.name.clone(), "1.0.0".to_string());
            process.description = job.description.clone();
            process
        })
        .collect::<Vec<_>>();
    /* Example:
    {
      "processes": [
        {
          "id": "EchoProcess",
          "title": "EchoProcess",
          "version": "1.0.0",
          "jobControlOptions": [
            "async-execute",
            "sync-execute"
          ],
          "outputTransmission": [
            "value",
            "reference"
          ],
          "additionalParameters": {
            "title": "string",
            "role": "string",
            "href": "string",
            "parameters": [
              {
                "name": "string",
                "value": [
                  "string",
                  0,
                  0,
                  [
                    null
                  ],
                  {}
                ]
              }
            ]
          },
          "links": [
            {
              "href": "https://processing.example.org/oapi-p/processes/EchoProcess",
              "type": "application/json",
              "rel": "self",
              "title": "process description"
            }
          ]
        }
      ],
      "links": [
        {
          "href": "https://processing.example.org/oapi-p/processes?f=json",
          "rel": "self",
          "type": "application/json"
        },
        {
          "href": "https://processing.example.org/oapi-p/processes?f=html",
          "rel": "alternate",
          "type": "text/html"
        }
      ]
    }
    */
    let resp = ProcessList {
        processes,
        links: Vec::new(),
    };

    HttpResponse::Ok().json(resp)
}

#[derive(OpenApi)]
#[openapi(handlers(processes), components(ProcessList))]
pub struct ApiDoc;

pub fn init_service(api: &mut OgcApiInventory, openapi: &mut utoipa::openapi::OpenApi) {
    api.landing_page_links.push(ApiLink {
        href: "/processes".to_string(),
        rel: Some("processes".to_string()),
        type_: Some("application/json".to_string()),
        title: Some("OGC API processes list".to_string()),
        hreflang: None,
        length: None,
    });
    api.conformance_classes.extend(vec![
        "http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/core".to_string(),
        "http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/ogc-process-description"
            .to_string(),
        "http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/json".to_string(),
        "http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/oas30".to_string(),
    ]);
    openapi.extend(ApiDoc::openapi());
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/processes").route(web::get().to(processes)));
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{body, dev::Service, http, test, App, Error};

    #[actix_web::test]
    async fn test_process_list() -> Result<(), Error> {
        let app = test::init_service(
            App::new().service(web::resource("/processes").route(web::get().to(processes))),
        )
        .await;

        let req = test::TestRequest::get().uri("/processes").to_request();
        let resp = app.call(req).await.unwrap();

        assert_eq!(resp.status(), http::StatusCode::OK);

        let response_body = body::to_bytes(resp.into_body()).await?;

        assert!(response_body.starts_with(b"{\"links\":["));

        Ok(())
    }
}
