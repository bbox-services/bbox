//! Backend for <https://dagster.io/>
//!
//! Term mapping (OGC -> Dagster):
//! * Process -> Job
//! * Job -> Run
//
// https://docs.dagster.io/concepts/dagit/graphql#using-the-graphql-api

use crate::error::{self, Result};
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

pub async fn get_process_description(job_name: &str) -> Result<serde_json::Value> {
    let variables = json!({"selector":{
            "repositoryName":"fpds2_processing_repository","repositoryLocationName":"fpds2_processing.repos",
            "pipelineName": job_name
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
    __typename
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
