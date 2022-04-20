use actix_web::{web, HttpRequest, HttpResponse};
use bbox_common::api::{ExtendApiDoc, OgcApiInventory};
use bbox_common::ogcapi::ApiLink;
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::{Component, OpenApi};

/// Information about the available processes
#[derive(Debug, Deserialize, Serialize, Component)]
pub struct Processes {}

/// retrieve the list of available processes
///
/// The list of processes contains a summary of each process the OGC API -
/// Processes offers, including the link to a more detailed description
/// of the process.
#[utoipa::path(
    post,
    path = "/processes",
    operation_id = "ProcessList",
    tag = "processes",
    responses(
        (status = 200, body = Processes),
    ),
)]
async fn processes(_req: HttpRequest) -> HttpResponse {
    let resp = json!({
      "processes": [
        {
          "title": "string",
          "description": "string",
          "keywords": [
            "string"
          ],
          "metadata": [
            {
              "title": "string",
              "role": "string",
              "href": "string"
            }
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
          "id": "string",
          "version": "string",
          "jobControlOptions": [
            "sync-execute"
          ],
          "outputTransmission": [
            [
              "value"
            ]
          ],
          "links": [
            {
              "href": "string",
              "rel": "service",
              "type": "application/json",
              "hreflang": "en",
              "title": "string"
            }
          ]
        }
      ],
      "links": [
        {
          "href": "string",
          "rel": "service",
          "type": "application/json",
          "hreflang": "en",
          "title": "string"
        }
      ]
    });

    HttpResponse::Ok().json(resp)
}

#[derive(OpenApi)]
#[openapi(handlers(processes), components())]
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

        assert_eq!(response_body, "{\"links\":[{\"href\":\"string\",\"hreflang\":\"en\",\"rel\":\"service\",\"title\":\"string\",\"type\":\"application/json\"}],\"processes\":[{\"additionalParameters\":{\"href\":\"string\",\"parameters\":[{\"name\":\"string\",\"value\":[\"string\",0,0,[null],{}]}],\"role\":\"string\",\"title\":\"string\"},\"description\":\"string\",\"id\":\"string\",\"jobControlOptions\":[\"sync-execute\"],\"keywords\":[\"string\"],\"links\":[{\"href\":\"string\",\"hreflang\":\"en\",\"rel\":\"service\",\"title\":\"string\",\"type\":\"application/json\"}],\"metadata\":[{\"href\":\"string\",\"role\":\"string\",\"title\":\"string\"}],\"outputTransmission\":[[\"value\"]],\"title\":\"string\",\"version\":\"string\"}]}");

        Ok(())
    }
}
