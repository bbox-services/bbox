//! Backend for <https://dagster.io/>
//!
//! Term mapping (OGC -> Dagster):
//! * Process -> Job
//! * Job -> Run
//
// https://docs.dagster.io/concepts/dagit/graphql#using-the-graphql-api

use crate::error::{self, Result};
use log::debug;
use serde::{Deserialize, Serialize};
use serde_json::json;

const GRAPHQL_URL: &str = "http://localhost:3000/graphql";

fn graphql_query(
    operation_name: &str,
    variables: serde_json::Value,
    query: &str,
) -> awc::SendClientRequest {
    let client = awc::Client::default();
    let request = json!({
        "operationName": operation_name,
        "body": "json",
        "variables": variables,
        "query": query
    });
    client.post(GRAPHQL_URL).send_json(&request)
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

pub async fn process_list() -> Result<Vec<Job>> {
    let variables = json!({"selector":{
            "repositoryName":"fpds2_processing_repository","repositoryLocationName":"fpds2_processing.repos"}});
    let mut response = graphql_query("JobsQuery", variables, JOBS_QUERY).await?;
    let resp: JobsQueryResponse = response.json().await?;
    Ok(resp.data.repository_or_error.jobs)
}
pub async fn get_process_description(process_id: &str) -> Result<serde_json::Value> {
    let variables = json!({"selector":{
            "repositoryName":"fpds2_processing_repository","repositoryLocationName":"fpds2_processing.repos",
            "pipelineName": process_id
    }});
    let mut response = graphql_query("OpSelectorQuery", variables, JOB_ARGS_QUERY).await?;
    Ok(response.json().await?)
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

pub async fn execute(job_name: &str, params: &Execute) -> Result<String> {
    let inputs = params.inputs.as_ref().map(|o| o.to_string());
    let variables = json!({
            "selector":{
                "repositoryName":"fpds2_processing_repository","repositoryLocationName":"fpds2_processing.repos",
                "jobName": job_name
            },
            "runConfigData": inputs
    });
    let mut response = graphql_query("LaunchRunMutation", variables, EXECUTE_JOB_QUERY).await?;
    // {"data":{"launchRun":{"run":{"runId":"d719c08f-d38e-4dbf-ac10-8fc3cf8412e3"}}}
    // {"data":{"launchRun":{}}
    // {"data":{"launchRun":{"errors":[{"message":"Received unexpected config entry \"XXXget_gemeinde_json\" at path root:ops. Expected: \"{ get_gemeinde_json: { config?: Any inputs: { fixpunkt_X: (Int | { json: { path: String } pickle: { path: String } value: Int }) fixpunkt_Y: (Int | { json: { path: String } pickle: { path: String } value: Int }) } outputs?: [{ result?: { json: { path: String } pickle: { path: String } } }] } }\".","reason":"FIELD_NOT_DEFINED"},{"message":"Missing required config entry \"get_gemeinde_json\" at path root:ops. Sample config for missing entry: {'get_gemeinde_json': {'inputs': {'fixpunkt_X': 0, 'fixpunkt_Y': 0}}}","reason":"MISSING_REQUIRED_FIELD"}]}}
    let resp = response.json::<LaunchRunResponse>().await;
    debug!("execute response: {resp:?}");
    match resp {
        Err(_) => Err(error::Error::BackendExecutionError(format!(
            "Process `{job_name}` not found"
        ))),
        Ok(resp) => match resp.data.launch_run {
            LaunchRun::Run { runId } => Ok(runId),
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

pub async fn get_jobs() -> Result<serde_json::Value> {
    let variables = json!({});
    let mut response = graphql_query("RunsQuery", variables, RUNS_QUERY).await?;
    Ok(response.json().await?)
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
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct RunResult {
    status: String,
    assets: Vec<AssetMaterializations>,
}

pub async fn get_status(job_id: &str) -> Result<String> {
    let variables = json!({ "runId": job_id });
    let mut response = graphql_query("FilteredRunsQuery", variables, FILTERED_RUNS_QUERY).await?;
    // {"data":{"runsOrError":{"results":[{"assets":[],"jobName":"create_db_schema_qwc","runId":"4a979b42-5831-4368-9913-685293a22ebc","stats":{"endTime":1654603294.525416,"startTime":1654603291.751443,"stepsFailed":1},"status":"FAILURE"}]}}}
    // {"data":{"runsOrError":{"results":[]}}}
    let resp = response.json::<FilteredRunResponse>().await;
    debug!("get_status response: {resp:?}");
    match resp {
        Err(_) => Err(error::Error::BackendExecutionError(format!(
            "Job `{job_id}` not found"
        ))),
        Ok(resp) => {
            let results = resp.data.runs_or_error.results;
            if results.len() == 1 {
                Ok(results[0].status.clone())
            } else {
                Err(error::Error::BackendExecutionError(format!(
                    "RunResult missing"
                )))
            }
        }
    }
}

pub async fn get_result(job_id: &str) -> Result<String> {
    let variables = json!({ "runId": job_id });
    let mut response = graphql_query("FilteredRunsQuery", variables, FILTERED_RUNS_QUERY).await?;
    // {"data":{"runsOrError":{"results":[{"assets":[{"assetMaterializations":[{"label":"get_gemeinde","metadataEntries":[{"jsonString":"{\"gemeinden\": [{\"bfs_nummer\": 770, \"gemeinde\": \"Stocken-H\\u00f6fen\", \"kanton\": \"BE\"}, {\"bfs_nummer\": 763, \"gemeinde\": \"Erlenbach im Simmental\", \"kanton\": \"BE\"}, {\"bfs_nummer\": 761, \"gemeinde\": \"D\\u00e4rstetten\", \"kanton\": \"BE\"}], \"lk_blatt\": 3451}"}]}],"id":"AssetKey(['get_gemeinde'])"}],"jobName":"get_gemeinde","runId":"c54ca13c-48ff-470e-8123-8f8f162208bd","stats":{"endTime":1654269774.36158,"startTime":1654269773.145739,"stepsFailed":0},"status":"SUCCESS"}]}}}
    // {"data":{"runsOrError":{"results":[{"assets":[{"assetMaterializations":[{"label":"import_fpds2_testdata_ili2pg_create_schema","metadataEntries":[{"label":"log_file_path","path":"/data/Interlis/KGKCGC_FPDS2_V1_0_ili2pg_create_schema.log"}]}],"id":"AssetKey(['import_fpds2_testdata_ili2pg_create_schema'])"},{"assetMaterializations":[{"label":"import_fpds2_testdata_ili2pg_import","metadataEntries":[{"label":"log_file_path","path":"/data/Interlis/Testdaten/DM_FPDS2_GR_ili2pg.log"}]}],"id":"AssetKey(['import_fpds2_testdata_ili2pg_import'])"}],"jobName":"import_fpds2_testdata","runId":"c1babe60-6f83-4122-a190-5c25f31a5c4d","stats":{"endTime":1653996610.788731,"startTime":1653996599.752907,"stepsFailed":0},"status":"SUCCESS"}]}}}
    let resp = response.json::<FilteredRunResponse>().await;
    debug!("get_result response: {resp:?}");
    match resp {
        Err(_) => Err(error::Error::BackendExecutionError(format!(
            "Job `{job_id}` not found"
        ))),
        Ok(resp) => {
            let results = resp.data.runs_or_error.results;
            if results.len() == 1 {
                if let Some(asset) = results[0].assets.get(0) {
                    if asset.asset_materializations.len() > 0 {
                        let entries = &asset.asset_materializations[0].metadata_entries;
                        if entries.len() > 0 {
                            if let Some(path) = &entries[0].path {
                                Ok(path.clone()) // TODO: return file content
                            } else if let Some(json_string) = &entries[0].json_string {
                                Ok(json_string.clone()) // TODO: convert to json
                            } else {
                                Err(error::Error::BackendExecutionError(format!(
                                    "Unknown metadata entry `{:?}`",
                                    entries[0]
                                )))
                            }
                        } else {
                            Err(error::Error::BackendExecutionError(format!(
                                "MetadataEntry missing"
                            )))
                        }
                    } else {
                        Err(error::Error::BackendExecutionError(format!(
                            "AssetMaterialization missing"
                        )))
                    }
                } else {
                    Err(error::Error::BackendExecutionError(format!(
                        "AssetMaterializations missing"
                    )))
                }
            } else {
                Err(error::Error::BackendExecutionError(format!(
                    "RunResult missing"
                )))
            }
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
        let jobs = process_list().await.unwrap();
        assert_eq!(jobs[0].name, "get_gemeinde");
        let job_args = get_process_description(&jobs[0].name).await;
        dbg!(&job_args);
    }
}
