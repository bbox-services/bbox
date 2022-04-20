use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::Component;

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

// #[derive(OpenApi)]
// #[openapi(
//     handlers(processes),
//     components(),
// )]
// pub struct ApiDoc;

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
