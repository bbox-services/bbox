use serde::{Deserialize, Serialize};

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
}

pub async fn query_jobs() -> Vec<Job> {
    let client = awc::Client::default();
    let query = r#"
        query JobsQuery {
          repositoryOrError(
            repositorySelector: {
              repositoryLocationName: "fpds2_processing.repos"
              repositoryName: "fpds2_processing_repository"
            }
          ) {
            ... on Repository {
              jobs {
                name
              }
            }
          }
        }"#;
    let request = serde_json::json!({
        "operationName":"JobsQuery",
        "body": "json",
        "variables":{},
        "query": query
    });
    let mut response = client
        .post("http://localhost:3000/graphql")
        .send_json(&request)
        .await
        .unwrap();
    let resp: JobsQueryResponse = response.json().await.unwrap();
    resp.data.repository_or_error.jobs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[actix_web::test]
    async fn query_test() {
        let jobs = query_jobs().await;
        assert_eq!(jobs[0].name, "get_gemeinde");
    }
}
