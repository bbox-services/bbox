//! Endpoints according to <https://ogcapi.ogc.org/processes/> API

use crate::dagster;
use crate::error;
use crate::models::*;
use actix_files::NamedFile;
use actix_web::{http::StatusCode, web, Either, HttpRequest, HttpResponse};
use bbox_common::api::{ExtendApiDoc, OgcApiInventory};
use bbox_common::ogcapi::ApiLink;
use log::{info, warn};
use serde_json::json;
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
async fn process_list(_req: HttpRequest) -> HttpResponse {
    let jobs = dagster::process_list().await.unwrap_or_else(|e| {
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
///
/// The process description contains information about inputs and outputs and a link to the execution-endpoint for the process. The Core does not mandate the use of a specific process description to specify the interface of a process.
// For more information, see [Section 7.10](https://docs.ogc.org/is/18-062/18-062.html#sc_process_description).
#[utoipa::path(
    get,
    path = "/processes/{processID}",
    operation_id = "getProcessDescription",
    tag = "ProcessDescription",
    responses(
        (status = 200),
    ),
)]
// parameters:
//   - $ref: "http://schemas.opengis.net/ogcapi/processes/part1/1.0/openapi/parameters/processIdPathParam.yaml"
// responses:
//   200:
//     $ref: "http://schemas.opengis.net/ogcapi/processes/part1/1.0/openapi/responses/ProcessDescription.yaml"
//   404:
//     $ref: "http://schemas.opengis.net/ogcapi/processes/part1/1.0/openapi/responses/NotFound.yaml"
async fn get_process_description(process_id: web::Path<String>) -> HttpResponse {
    match dagster::get_process_description(&process_id).await {
        Ok(descr) => HttpResponse::Ok().json(descr), // TODO: type ProcessDescription
        Err(error::Error::NotFound(type_)) => HttpResponse::NotFound().json(Exception::new(type_)),
        Err(e) => HttpResponse::InternalServerError().json(Exception::from(e)),
    }
}

/// execute a process
///
/// Create a new job.
// For more information, see [Section 7.11](https://docs.ogc.org/is/18-062/18-062.html#sc_create_job).
#[utoipa::path(
    post,
    path = "/processes/{processID}/execution",
    operation_id = "execute",
    tag = "Execute",
    responses(
        (status = 200),
    ),
)]
async fn execute(
    process_id: web::Path<String>,
    parameters: web::Json<dagster::Execute>,
) -> HttpResponse {
    info!("Execute `{process_id}` with parameters `{parameters:?}`");
    match dagster::execute(&process_id, &*parameters).await {
        /* responses:
                200:
                  $ref: 'http://schemas.opengis.net/ogcapi/processes/part1/1.0/openapi/responses/ExecuteSync.yaml'
        */
        Ok(status) => HttpResponse::build(StatusCode::CREATED).json(status),
        Err(error::Error::NotFound(type_)) => HttpResponse::NotFound().json(Exception::new(type_)),
        Err(e) => HttpResponse::InternalServerError().json(Exception::from(e)),
    }
}

/// retrieve the list of jobs
///
/// Lists available jobs.
// For more information, see [Section 12](http://docs.ogc.org/DRAFTS/18-062.html#Job_list).
#[utoipa::path(
    get,
    path = "/jobs",
    operation_id = "getJobs",
    tag = "JobList",
    responses(
        (status = 200),
    ),
)]
// responses:
//   200:
//     $ref: "https://raw.githubusercontent.com/opengeospatial/ogcapi-processes/master/core/openapi/responses/JobList.yaml"
//   404:
//     $ref: "https://raw.githubusercontent.com/opengeospatial/ogcapi-processes/master/core/openapi/responses/NotFound.yaml"
async fn get_jobs(_req: HttpRequest) -> HttpResponse {
    match dagster::get_jobs().await {
        Ok(jobs) => HttpResponse::Ok().json(jobs), // TODO: type JobList
        Err(e) => HttpResponse::InternalServerError().json(Exception::from(e)),
    }
}

/// retrieve the status of a job
///
/// Shows the status of a job.
// For more information, see [Section 7.10](http://docs.ogc.org/DRAFTS/18-062.html#sc_retrieve_status_info).
#[utoipa::path(
    get,
    path = "/jobs/{jobId}",
    operation_id = "getStatus",
    tag = "Status",
    responses(
        (status = 200),
    ),
)]
async fn get_status(job_id: web::Path<String>) -> HttpResponse {
    match dagster::get_status(&job_id).await {
        Ok(status) => HttpResponse::Ok().json(status),
        Err(error::Error::NotFound(type_)) => HttpResponse::NotFound().json(Exception::new(type_)),
        Err(e) => HttpResponse::InternalServerError().json(Exception::from(e)),
    }
}

/// cancel a job execution, remove a finished job
///
/// Cancel a job execution and remove it from the jobs list.
// For more information, see [Section 14](http://docs.ogc.org/DRAFTS/18-062.html#Dismiss).
#[utoipa::path(
    delete,
    path = "/jobs/{jobId}",
    operation_id = "dismiss",
    tag = "Dismiss",
    responses(
        (status = 200),
    ),
)]
// parameters:
//   - $ref: "https://raw.githubusercontent.com/opengeospatial/ogcapi-processes/master/core/openapi/parameters/jobID.yaml"
// responses:
//   200:
//     $ref: "https://raw.githubusercontent.com/opengeospatial/ogcapi-processes/master/core/openapi/responses/Status.yaml"
//   404:
//     $ref: "https://raw.githubusercontent.com/opengeospatial/ogcapi-processes/master/core/openapi/responses/NotFound.yaml"
//   500:
//     $ref: "https://raw.githubusercontent.com/opengeospatial/ogcapi-processes/master/core/openapi/responses/ServerError.yaml"
async fn dismiss(job_id: web::Path<String>) -> HttpResponse {
    HttpResponse::InternalServerError().json(job_id.to_string())
}

pub enum JobResult {
    FilePath(String),
    Json(serde_json::Value),
}

type JobResultResponse = Either<HttpResponse, std::result::Result<NamedFile, std::io::Error>>;

/// retrieve the result(s) of a job
///
/// Lists available results of a job. In case of a failure, lists exceptions instead.
// For more information, see [Section 7.11](http://docs.ogc.org/DRAFTS/18-062.html#sc_retrieve_job_results).
#[utoipa::path(
    get,
    path = "/jobs/{jobId}/results",
    operation_id = "getResult",
    tag = "Result",
    responses(
        (status = 200),
    ),
)]
async fn get_result(job_id: web::Path<String>) -> JobResultResponse {
    match dagster::get_result(&job_id).await {
        Ok(result) => match result {
            JobResult::Json(json) => {
                // qualifiedInputValue
                Either::Left(HttpResponse::Ok().json(json!({ "result": {"value": json }})))
            }
            JobResult::FilePath(path) => {
                info!("get_result from {path}");
                Either::Right(NamedFile::open(path))
            }
        },
        Err(error::Error::NotFound(type_)) => {
            Either::Left(HttpResponse::NotFound().json(Exception::new(type_)))
        }
        Err(e) => Either::Left(HttpResponse::InternalServerError().json(Exception::from(e))),
    }
}

#[derive(OpenApi)]
#[openapi(
    handlers(process_list, execute, get_jobs, get_status, dismiss, get_result),
    components(ProcessList)
)]
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
        // "http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/ogc-process-description"
        //     .to_string(),
        "http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/json".to_string(),
        "http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/oas30".to_string(),
    ]);
    openapi.extend(ApiDoc::openapi());
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/processes").route(web::get().to(process_list)))
        .service(
            web::resource("/processes/{processID}").route(web::get().to(get_process_description)),
        )
        .service(web::resource("/processes/{processID}/execution").route(web::post().to(execute)))
        .service(web::resource("/jobs").route(web::get().to(get_jobs)))
        .service(web::resource("/jobs/{jobId}").route(web::get().to(get_status)))
        .service(web::resource("/jobs/{jobId}").route(web::delete().to(dismiss)))
        .service(web::resource("/jobs/{jobId}/results").route(web::get().to(get_result)));
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
