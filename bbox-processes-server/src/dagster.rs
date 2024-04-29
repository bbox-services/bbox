//! Backend for <https://dagster.io/>
//!
//! Term mapping (OGC -> Dagster):
//! * Process -> Job
//! * Job -> Run
//
// https://docs.dagster.io/concepts/dagit/graphql#using-the-graphql-api

use crate::config::{DagsterBackendCfg, ProcessesServiceCfg};
use crate::endpoints::JobResult;
use crate::error::{self, Result};
use crate::models::{self, StatusCode};
use log::debug;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;

#[derive(Clone)]
pub struct DagsterBackend {
    config: DagsterBackendCfg,
}

impl DagsterBackend {
    pub fn new() -> Self {
        let config = ProcessesServiceCfg::from_config()
            .dagster_backend
            .expect("Backend config missing");
        DagsterBackend { config }
    }
    fn graphql_query(
        &self,
        operation_name: &str,
        variables: serde_json::Value,
        query: &str,
    ) -> awc::SendClientRequest {
        let client = awc::Client::builder()
            .timeout(Duration::from_millis(
                self.config.request_timeout.unwrap_or(10000),
            ))
            .finish();
        let request = json!({
            "operationName": operation_name,
            "body": "json",
            "variables": variables,
            "query": query
        });
        client.post(&self.config.graphql_url).send_json(&request)
    }
}

impl Default for DagsterBackend {
    fn default() -> Self {
        Self::new()
    }
}

// --- process_list ---

#[derive(Serialize, Deserialize, Debug)]
struct JobsQueryResponse {
    data: JobsQueryResponseData,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct JobsQueryResponseData {
    repository_or_error: RepositoryOrError,
}

#[derive(Serialize, Deserialize, Debug)]
struct RepositoryOrError {
    jobs: Vec<Job>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Job {
    pub name: String,
    pub description: Option<String>,
}

impl DagsterBackend {
    pub async fn process_list(&self) -> Result<Vec<Job>> {
        let variables = json!({"selector":{
            "repositoryName": &self.config.repository_name,
            "repositoryLocationName": &self.config.repository_location_name
        }});
        let mut response = self
            .graphql_query("JobsQuery", variables, JOBS_QUERY)
            .await?;
        let resp: JobsQueryResponse = response.json().await?;
        Ok(resp.data.repository_or_error.jobs)
    }
    pub async fn get_process_description(&self, process_id: &str) -> Result<serde_json::Value> {
        let variables = json!({"selector":{
                "repositoryName": &self.config.repository_name,
                "repositoryLocationName": &self.config.repository_location_name,
                "pipelineName": process_id
        }});
        let mut response = self
            .graphql_query("OpSelectorQuery", variables, JOB_ARGS_QUERY)
            .await?;
        Ok(response.json().await?)
    }
}

// --- execute ---

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Execute {
    pub inputs: Option<serde_json::Value>,
    pub outputs: Option<serde_json::Value>,
    pub response: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct LaunchRunResponse {
    data: LaunchRunResponseData,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct LaunchRunResponseData {
    launch_run: LaunchRun,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
enum LaunchRun {
    #[allow(non_snake_case)]
    Run {
        runId: String,
    },
    Errors(Vec<ErrorMessage>),
}

#[derive(Serialize, Deserialize, Debug)]
struct ErrorMessage {
    message: String,
}

impl DagsterBackend {
    pub async fn execute(&self, process_id: &str, params: &Execute) -> Result<models::StatusInfo> {
        let inputs = params.inputs.as_ref().map(|o| o.to_string());
        let variables = json!({
                "selector":{
                    "repositoryName": &self.config.repository_name,
                    "repositoryLocationName": &self.config.repository_location_name,
                    "jobName": process_id
                },
                "runConfigData": inputs
        });
        let mut response = self
            .graphql_query("LaunchRunMutation", variables, EXECUTE_JOB_QUERY)
            .await?;
        // {"data":{"launchRun":{"run":{"runId":"d719c08f-d38e-4dbf-ac10-8fc3cf8412e3"}}}
        // {"data":{"launchRun":{}}
        // {"data":{"launchRun":{"errors":[{"message":"Received unexpected config entry \"XXXget_gemeinde_json\" at path root:ops. Expected: \"{ get_gemeinde_json: { config?: Any inputs: { fixpunkt_X: (Int | { json: { path: String } pickle: { path: String } value: Int }) fixpunkt_Y: (Int | { json: { path: String } pickle: { path: String } value: Int }) } outputs?: [{ result?: { json: { path: String } pickle: { path: String } } }] } }\".","reason":"FIELD_NOT_DEFINED"},{"message":"Missing required config entry \"get_gemeinde_json\" at path root:ops. Sample config for missing entry: {'get_gemeinde_json': {'inputs': {'fixpunkt_X': 0, 'fixpunkt_Y': 0}}}","reason":"MISSING_REQUIRED_FIELD"}]}}
        let resp = response.json::<LaunchRunResponse>().await;
        debug!("execute response: {resp:?}");
        match resp {
            Err(_) => Err(error::Error::NotFound(
                "http://www.opengis.net/def/exceptions/ogcapi-processes-1/1.0/no-such-process"
                    .to_string(),
            )),
            Ok(resp) => match resp.data.launch_run {
                LaunchRun::Run { runId } => {
                    let status = models::StatusInfo::new(
                        "process".to_string(),
                        runId,
                        models::StatusCode::ACCEPTED,
                    );
                    Ok(status)
                }
                LaunchRun::Errors(messages) => Err(error::Error::BackendExecutionError(
                    messages
                        .iter()
                        .map(|m| m.message.clone())
                        .collect::<Vec<_>>()
                        .join(" "),
                )),
            },
        }
    }

    pub async fn execute_sync(&self, process_id: &str, params: &Execute) -> Result<JobResult> {
        let mut status_info = self.execute(process_id, params).await?;
        let job_id = status_info.job_id.clone();
        while status_info.status == StatusCode::ACCEPTED
            || status_info.status == StatusCode::RUNNING
        {
            status_info = self.get_status(&job_id).await?;
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        if status_info.status != StatusCode::SUCCESSFUL {
            return Err(error::Error::BackendExecutionError(
                status_info.message.unwrap_or(format!(
                    "Job {job_id} failed (Status `{}`)",
                    status_info.status
                )),
            ));
        }
        self.get_result(&job_id).await
    }

    pub async fn get_jobs(&self) -> Result<serde_json::Value> {
        let variables = json!({});
        let mut response = self
            .graphql_query("RunsQuery", variables, RUNS_QUERY)
            .await?;
        Ok(response.json().await?)
    }
}

// --- get_status ---

#[derive(Serialize, Deserialize, Debug)]
struct FilteredRunResponse {
    data: FilteredRunResponseData,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct FilteredRunResponseData {
    runs_or_error: RunsOrError,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct RunsOrError {
    results: Vec<RunResult>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct AssetMaterializations {
    asset_materializations: Vec<AssetMaterialization>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct AssetMaterialization {
    label: String,
    metadata_entries: Vec<MetadataEntry>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct MetadataEntry {
    label: String,
    path: Option<String>,
    json_string: Option<String>,
    text: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct EventConnection {
    #[serde(default)]
    events: Vec<Event>,
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Event {
    #[serde(rename = "__typename")]
    typename: String,
    message: Option<String>,
    step_key: Option<String>,
    failure_metadata: Option<FailureMetadata>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct FailureMetadata {
    description: String,
    metadata_entries: Vec<MetadataEntry>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct RunResult {
    status: String,
    #[serde(default)]
    assets: Vec<AssetMaterializations>,
    event_connection: EventConnection,
}

impl DagsterBackend {
    pub async fn get_status(&self, job_id: &str) -> Result<models::StatusInfo> {
        let variables = json!({ "runId": job_id });
        let mut response = self
            .graphql_query("FilteredRunsQuery", variables, FILTERED_RUNS_QUERY)
            .await?;
        // {"data":{"runsOrError":{"results":[{"assets":[],"jobName":"create_db_schema_qwc","runId":"4a979b42-5831-4368-9913-685293a22ebc","stats":{"endTime":1654603294.525416,"startTime":1654603291.751443,"stepsFailed":1},"status":"FAILURE"}]}}}
        // {"data":{"runsOrError":{"results":[]}}}
        let resp = response.json::<FilteredRunResponse>().await;
        debug!("get_status response: {resp:?}");
        match resp {
            Err(e) => Err(error::Error::BackendExecutionError(e.to_string())),
            Ok(resp) => {
                let results = &resp.data.runs_or_error.results;
                if results.len() == 1 {
                    let status = match results[0].status.as_str() {
                        "QUEUED" | "NOT_STARTED" | "MANAGED" | "STARTING" => StatusCode::ACCEPTED,
                        "STARTED" => StatusCode::RUNNING,
                        "SUCCESS" => StatusCode::SUCCESSFUL,
                        "FAILURE" => StatusCode::FAILED,
                        "CANCELING" | "CANCELED" => StatusCode::DISMISSED,
                        _ => StatusCode::DISMISSED,
                    };
                    let mut status =
                        models::StatusInfo::new("process".to_string(), job_id.to_string(), status);
                    if let Some((message, description, error)) = self.extract_error_message(&resp) {
                        let message = [message, description, error].join(" - ");
                        status.message = Some(message);
                    }
                    Ok(status)
                } else {
                    Err(error::Error::NotFound(
                        "http://www.opengis.net/def/exceptions/ogcapi-processes-1/1.0/no-such-job"
                            .to_string(),
                    ))
                }
            }
        }
    }

    pub async fn get_result(&self, job_id: &str) -> Result<JobResult> {
        let variables = json!({ "runId": job_id });
        let mut response = self
            .graphql_query("FilteredRunsQuery", variables, FILTERED_RUNS_QUERY)
            .await?;
        // {"data":{"runsOrError":{"results":[{"assets":[{"assetMaterializations":[{"label":"get_gemeinde","metadataEntries":[{"jsonString":"{\"gemeinden\": [{\"bfs_nummer\": 770, \"gemeinde\": \"Stocken-H\\u00f6fen\", \"kanton\": \"BE\"}, {\"bfs_nummer\": 763, \"gemeinde\": \"Erlenbach im Simmental\", \"kanton\": \"BE\"}, {\"bfs_nummer\": 761, \"gemeinde\": \"D\\u00e4rstetten\", \"kanton\": \"BE\"}], \"lk_blatt\": 3451}"}]}],"id":"AssetKey(['get_gemeinde'])"}],"jobName":"get_gemeinde","runId":"c54ca13c-48ff-470e-8123-8f8f162208bd","stats":{"endTime":1654269774.36158,"startTime":1654269773.145739,"stepsFailed":0},"status":"SUCCESS"}]}}}
        // {"data":{"runsOrError":{"results":[{"assets":[{"assetMaterializations":[{"label":"import_fpds2_testdata_ili2pg_create_schema","metadataEntries":[{"label":"log_file_path","path":"/data/Interlis/KGKCGC_FPDS2_V1_0_ili2pg_create_schema.log"}]}],"id":"AssetKey(['import_fpds2_testdata_ili2pg_create_schema'])"},{"assetMaterializations":[{"label":"import_fpds2_testdata_ili2pg_import","metadataEntries":[{"label":"log_file_path","path":"/data/Interlis/Testdaten/DM_FPDS2_GR_ili2pg.log"}]}],"id":"AssetKey(['import_fpds2_testdata_ili2pg_import'])"}],"jobName":"import_fpds2_testdata","runId":"c1babe60-6f83-4122-a190-5c25f31a5c4d","stats":{"endTime":1653996610.788731,"startTime":1653996599.752907,"stepsFailed":0},"status":"SUCCESS"}]}}}
        let resp = response.json::<FilteredRunResponse>().await;
        debug!("get_result response: {resp:?}");
        match resp {
            Err(e) => Err(error::Error::BackendExecutionError(e.to_string())),
            Ok(resp) => {
                if let Some(result) = resp.data.runs_or_error.results.first() {
                    if let Some(asset) = result.assets.last() {
                        if !asset.asset_materializations.is_empty() {
                            if let Some(meta) =
                                &asset.asset_materializations[0].metadata_entries.first()
                            {
                                if let Some(path) = &meta.path {
                                    Ok(JobResult::FilePath(path.clone()))
                                } else if let Some(json_string) = &meta.json_string {
                                    Ok(JobResult::Json(serde_json::from_str(json_string)?))
                                } else {
                                    Err(error::Error::BackendExecutionError(format!(
                                        "Unknown metadata entry `{:?}`",
                                        meta
                                    )))
                                }
                            } else {
                                Err(error::Error::BackendExecutionError(
                                    "MetadataEntry missing".to_string(),
                                ))
                            }
                        } else {
                            Err(error::Error::BackendExecutionError(
                                "AssetMaterialization missing".to_string(),
                            ))
                        }
                    } else if let Some((message, description, error)) =
                        self.extract_error_message(&resp)
                    {
                        Ok(JobResult::Json(json!({
                            "message": message,
                            "description": description,
                            "error": error,
                        })))
                    } else {
                        // No Assets and no ExecutionStepFailureEvent
                        Ok(JobResult::Json(json!(result.status)))
                    }
                } else {
                    Err(error::Error::NotFound(
                        "http://www.opengis.net/def/exceptions/ogcapi-processes-1/1.0/no-such-job"
                            .to_string(),
                    ))
                }
            }
        }
    }

    /// Extract message, description and error from failure event
    fn extract_error_message(
        &self,
        resp: &FilteredRunResponse,
    ) -> Option<(String, String, String)> {
        if let Some(result) = resp.data.runs_or_error.results.first() {
            // Extract failure messages
            if let Some(event) = result
                .event_connection
                .events
                .iter()
                .find(|ev| ev.typename == "ExecutionStepFailureEvent")
            {
                let message = event
                    .message
                    .as_ref()
                    .unwrap_or(&"No FailureEvent message".to_string())
                    .clone();
                if let Some(meta) = &event.failure_metadata {
                    // meta.metadata_entries[0].label == "output"
                    let error = meta.metadata_entries[0]
                        .text
                        .as_ref()
                        .unwrap_or(&"No FailureEvent Metadata text".to_string())
                        .clone();
                    Some((message, meta.description.clone(), error))
                } else {
                    Some((message, "".to_string(), "".to_string()))
                }
            } else {
                // No ExecutionStepFailureEvent
                None
            }
        } else {
            None
        }
    }
}

/// Get list of jobs in repository
// Example variables: {"selector":{"repositoryName":"fpds2_processing_repository","repositoryLocationName":"fpds2_processing.repos"}}
const JOBS_QUERY: &str = r#"
query JobsQuery($selector: RepositorySelector!) {
  repositoryOrError(repositorySelector: $selector) {
    ... on Repository {
      jobs {
        id
        name
        description
        tags {
          key
          value
        }
      }
    }
  }
}"#;

/// Get inputs&output of a job
// Example variables: {"selector":{"repositoryName":"fpds2_processing_repository","repositoryLocationName":"fpds2_processing.repos","pipelineName":"get_gemeinde"},"requestScopeHandleID":""}
const JOB_ARGS_QUERY: &str = r#"
query OpSelectorQuery($selector: PipelineSelector!, $requestScopeHandleID: String) {
  pipelineOrError(params: $selector) {
    __typename
    ... on Pipeline {
      id
      name
      solidHandles(parentHandleID: $requestScopeHandleID) {
        handleID
        solid {
          name
          __typename
        }
        ...GraphExplorerSolidHandleFragment
        __typename
      }
      __typename
    }
    ... on PipelineNotFoundError {
      message
      __typename
    }
    ... on InvalidSubsetError {
      message
      __typename
    }
    ... on PythonError {
      message
      __typename
    }
  }
  }
fragment GraphExplorerSolidHandleFragment on SolidHandle {
  handleID
  solid {
    name
    ...PipelineGraphOpFragment
    __typename
  }
  __typename
  }
fragment PipelineGraphOpFragment on Solid {
  name
  ...OpNodeInvocationFragment
  definition {
    name
    ...OpNodeDefinitionFragment
    __typename
  }
  __typename
  }
fragment OpNodeInvocationFragment on Solid {
  name
  isDynamicMapped
  inputs {
    definition {
      name
      __typename
    }
    isDynamicCollect
    dependsOn {
      definition {
        name
        type {
          displayName
          __typename
        }
        __typename
      }
      solid {
        name
        __typename
      }
      __typename
    }
    __typename
  }
  outputs {
    definition {
      name
      __typename
    }
    dependedBy {
      solid {
        name
        __typename
      }
      definition {
        name
        type {
          displayName
          __typename
        }
        __typename
      }
      __typename
    }
    __typename
  }
  __typename
  }
fragment OpNodeDefinitionFragment on ISolidDefinition {
  __typename
  name
  description
  metadata {
    key
    value
    __typename
  }
  assetNodes {
    id
    assetKey {
      path
      __typename
    }
    __typename
  }
  inputDefinitions {
    name
    type {
      displayName
      __typename
    }
    __typename
  }
  outputDefinitions {
    name
    isDynamic
    type {
      displayName
      __typename
    }
    __typename
  }
  ... on SolidDefinition {
    configField {
      configType {
        key
        description
        __typename
      }
      __typename
    }
    __typename
  }
  ... on CompositeSolidDefinition {
    id
    inputMappings {
      definition {
        name
        __typename
      }
      mappedInput {
        definition {
          name
          __typename
        }
        solid {
          name
          __typename
        }
        __typename
      }
      __typename
    }
    outputMappings {
      definition {
        name
        __typename
      }
      mappedOutput {
        definition {
          name
          __typename
        }
        solid {
          name
          __typename
        }
        __typename
      }
      __typename
    }
    __typename
  }
}"#;

/// Execute a job
// Example variables: {"selector":{"repositoryName":"fpds2_processing_repository","repositoryLocationName":"fpds2_processing.repos","jobName":"get_gemeinde"},"runConfigData": "{\"ops\": {\"get_gemeinde_json\": {\"inputs\": {\"fixpunkt_X\": 2607545, \"fixpunkt_Y\": 1171421}}}}"}
const EXECUTE_JOB_QUERY: &str = r#"
mutation LaunchRunMutation($selector: JobOrPipelineSelector!, $runConfigData: RunConfigData) {
  launchRun(
    executionParams: {
      selector: $selector
      runConfigData: $runConfigData
    }
  ) {
    ... on LaunchRunSuccess {
      run {
        runId
      }
    }
    ... on RunConfigValidationInvalid {
      errors {
        message
        reason
      }
    }
    ... on PythonError {
      message
    }
  }
}"#;

const RUNS_QUERY: &str = r#"
query RunsQuery {
  runsOrError {
    ... on Runs {
      results {
        runId
        jobName
        status
        runConfigYaml
        stats {
          ... on RunStatsSnapshot {
            startTime
            endTime
            stepsFailed
          }
        }
      }
    }
  }
}"#;

const FILTERED_RUNS_QUERY: &str = r#"
query FilteredRunsQuery($runId: String) {
  runsOrError(filter: { runIds: [$runId] }) {
    ... on Runs {
      results {
        runId
        jobName
        status
        assets {
          ... on Asset {
            id
            assetMaterializations(limit: 1) {
            ... on MaterializationEvent {
              label
              metadataEntries {
                label
                ... on PathMetadataEntry {
                  path
                }
                ... on JsonMetadataEntry {
                  jsonString
                }
              }
            }
            }
          }
        }
        eventConnection {
          ... on EventConnection {
            events {
              __typename
              ... on ExecutionStepFailureEvent {
                message
                stepKey
                failureMetadata {
                  description
                  metadataEntries {
                    label
                    ... on TextMetadataEntry {
                      text
                    }
                  }
                }
              }
            }
          }
        }
        stats {
          ... on RunStatsSnapshot {
            startTime
            endTime
            stepsFailed
          }
        }
      }
    }
  }
}"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[actix_web::test]
    #[ignore]
    async fn query_test() {
        if ProcessesServiceCfg::from_config().dagster_backend.is_none() {
            return;
        }
        let backend = DagsterBackend::new();
        let jobs = backend.process_list().await.unwrap();
        assert_eq!(jobs[0].name, "get_gemeinde");
        let job_args = backend.get_process_description(&jobs[0].name).await;
        dbg!(&job_args);
    }
}
