use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize, Debug)]
struct JobsQueryResponse {
    data: Data,
}

#[derive(Serialize, Deserialize, Debug)]
struct Data {
    #[serde(rename = "repositoryOrError")]
    repository_or_error: RepositoryOrError,
}

#[derive(Serialize, Deserialize, Debug)]
struct RepositoryOrError {
    jobs: Vec<Job>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Job {
    name: String,
    description: Option<String>,
}

const GRAPHQL_URL: &str = "http://localhost:3000/graphql";

pub async fn query_jobs() -> Vec<Job> {
    let client = awc::Client::default();
    let request = json!({
        "operationName":"JobsQuery",
        "body": "json",
        "variables": {"selector":{
            "repositoryName":"fpds2_processing_repository","repositoryLocationName":"fpds2_processing.repos"}},
        "query": JOBS_QUERY
    });
    let mut response = client.post(GRAPHQL_URL).send_json(&request).await.unwrap();
    let resp: JobsQueryResponse = response.json().await.unwrap();
    resp.data.repository_or_error.jobs
}

pub async fn query_job_args(job_name: &str) -> serde_json::Value {
    let client = awc::Client::default();
    let request = json!({
        "operationName":"OpSelectorQuery",
        "body": "json",
        "variables": {"selector":{
            "repositoryName":"fpds2_processing_repository","repositoryLocationName":"fpds2_processing.repos",
            "pipelineName": job_name
        }},
        "query": JOB_ARGS_QUERY
    });
    let mut response = client.post(GRAPHQL_URL).send_json(&request).await.unwrap();
    response.json().await.unwrap()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[actix_web::test]
    async fn query_test() {
        let jobs = query_jobs().await;
        assert_eq!(jobs[0].name, "get_gemeinde");
        let job_args = query_job_args(&jobs[0].name).await;
        dbg!(&job_args);
    }
}
