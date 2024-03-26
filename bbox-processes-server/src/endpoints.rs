//! Endpoints according to <https://ogcapi.ogc.org/processes/> API

use crate::dagster::{self, DagsterBackend};
use crate::error;
use crate::models::*;
use crate::service::ProcessesService;
use actix_files::NamedFile;
use actix_web::{
    http::header::ContentEncoding, http::StatusCode, web, Either, HttpRequest, HttpResponse,
};
use bbox_core::service::ServiceEndpoints;
use log::{info, warn};
use serde_json::json;

/// retrieve the list of available processes
async fn process_list(_req: HttpRequest) -> HttpResponse {
    let backend = DagsterBackend::new();
    let jobs = backend.process_list().await.unwrap_or_else(|e| {
        warn!("Dagster backend error: {e}");
        Vec::new()
    });
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

/// retrieve a process description
async fn get_process_description(process_id: web::Path<String>) -> HttpResponse {
    let backend = DagsterBackend::new();
    match backend.get_process_description(&process_id).await {
        Ok(descr) => HttpResponse::Ok().json(descr), // TODO: type ProcessDescription
        Err(error::Error::NotFound(type_)) => HttpResponse::NotFound().json(Exception::new(type_)),
        Err(e) => HttpResponse::InternalServerError().json(Exception::from(e)),
    }
}

/// execute a process
async fn execute(
    process_id: web::Path<String>,
    parameters: web::Json<dagster::Execute>,
    req: HttpRequest,
) -> JobResultResponse {
    info!("Execute `{process_id}` with parameters `{parameters:?}`");
    let backend = DagsterBackend::new();
    let prefer_async = req
        .headers()
        .get("Prefer")
        .and_then(|headerval| {
            headerval
                .to_str()
                .ok()
                .map(|headerstr| headerstr.contains("respond-async"))
        })
        .unwrap_or(false);
    // TODO: support sync/async-only processes
    if prefer_async {
        let resp = match backend.execute(&process_id, &parameters).await {
            /* responses:
                    200:
                      $ref: 'http://schemas.opengis.net/ogcapi/processes/part1/1.0/openapi/responses/ExecuteSync.yaml'
            */
            Ok(status) => HttpResponse::build(StatusCode::CREATED).json(status),
            Err(error::Error::NotFound(type_)) => {
                HttpResponse::NotFound().json(Exception::new(type_))
            }
            Err(e) => HttpResponse::InternalServerError().json(Exception::from(e)),
        };
        Either::Left(resp)
    } else {
        let job_result = backend.execute_sync(&process_id, &parameters).await;
        // TODO: respect parameters.response != "raw"
        job_result_response(job_result)
    }
}

/// retrieve the list of jobs
async fn get_jobs() -> HttpResponse {
    let backend = DagsterBackend::new();
    match backend.get_jobs().await {
        Ok(jobs) => HttpResponse::Ok().json(jobs), // TODO: type JobList
        Err(e) => HttpResponse::InternalServerError().json(Exception::from(e)),
    }
}

/// retrieve the status of a job
async fn get_status(job_id: web::Path<String>) -> HttpResponse {
    let backend = DagsterBackend::new();
    match backend.get_status(&job_id).await {
        Ok(status) => HttpResponse::Ok().json(status),
        Err(error::Error::NotFound(type_)) => HttpResponse::NotFound().json(Exception::new(type_)),
        Err(e) => HttpResponse::InternalServerError().json(Exception::from(e)),
    }
}

/// cancel a job execution, remove a finished job
async fn dismiss(job_id: web::Path<String>) -> HttpResponse {
    HttpResponse::InternalServerError().json(job_id.to_string())
}

pub enum JobResult {
    FilePath(String),
    Json(serde_json::Value),
}

type JobResultResponse = Either<HttpResponse, std::result::Result<NamedFile, std::io::Error>>;

/// retrieve the result(s) of a job
async fn get_result(job_id: web::Path<String>) -> JobResultResponse {
    let backend = DagsterBackend::new();
    let job_result = backend.get_result(&job_id).await;
    job_result_response(job_result)
}

fn job_result_response(job_result: crate::error::Result<JobResult>) -> JobResultResponse {
    match job_result {
        Ok(result) => match result {
            JobResult::Json(json) => {
                // qualifiedInputValue
                Either::Left(HttpResponse::Ok().json(json!({ "result": {"value": json }})))
            }
            JobResult::FilePath(path) => {
                info!("get_result from {path}");
                // Prevent file compression for now.
                // With compression enabled, files are returned compressed with the following headers:
                // * content-encoding: gzip
                // * vary: accept-encoding
                // * content-type: application/pdf
                // * content-disposition: attachment; filename="12575280.pdf"
                // This seems correct, but clients don't decompress the attached file!?
                let file = NamedFile::open(path)
                    .unwrap()
                    .set_content_encoding(ContentEncoding::Identity);
                Either::Right(Ok(file))
            }
        },
        Err(error::Error::NotFound(type_)) => {
            Either::Left(HttpResponse::NotFound().json(Exception::new(type_)))
        }
        Err(e) => Either::Left(HttpResponse::InternalServerError().json(Exception::from(e))),
    }
}

impl ServiceEndpoints for ProcessesService {
    fn register_endpoints(&self, cfg: &mut web::ServiceConfig) {
        if self.backend.is_none() {
            return;
        }
        cfg.service(web::resource("/processes").route(web::get().to(process_list)))
            .service(
                web::resource("/processes/{processID}")
                    .route(web::get().to(get_process_description)),
            )
            .service(
                web::resource("/processes/{processID}/execution").route(web::post().to(execute)),
            )
            .service(web::resource("/jobs").route(web::get().to(get_jobs)))
            .service(web::resource("/jobs/{jobId}").route(web::get().to(get_status)))
            .service(web::resource("/jobs/{jobId}").route(web::delete().to(dismiss)))
            .service(web::resource("/jobs/{jobId}/results").route(web::get().to(get_result)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{body, dev::Service, http, test, App, Error};

    #[actix_web::test]
    #[ignore]
    async fn test_process_list() -> Result<(), Error> {
        let app = test::init_service(
            App::new().service(web::resource("/processes").route(web::get().to(process_list))),
        )
        .await;

        let req = test::TestRequest::get().uri("/processes").to_request();
        let resp = app.call(req).await.unwrap();

        assert_eq!(resp.status(), http::StatusCode::OK);

        let response_body = body::to_bytes(resp.into_body()).await?;
        println!("{response_body:?}");
        assert!(response_body.starts_with(b"{\"processes\":["));

        Ok(())
    }
}
