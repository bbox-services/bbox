//! Error and Result types.
use crate::models;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    // Backend errors
    #[error(transparent)]
    BackendSendError(#[from] awc::error::SendRequestError),
    #[error(transparent)]
    BackendResponseError(#[from] awc::error::JsonPayloadError),
    #[error(transparent)]
    BackendJsonError(#[from] serde_json::Error),
    #[error("Backend execution error - {0}")]
    BackendExecutionError(String),
    // General
    #[error("I/O error")]
    IoError(#[from] std::io::Error),
    // OGC Error responses
    #[error("Resource not found - `{0}`")]
    NotFound(String),
}

pub type Result<T> = std::result::Result<T, Error>;

// Convert to OGC exception response
impl From<Error> for models::Exception {
    fn from(error: Error) -> Self {
        match error {
            Error::NotFound(type_) => models::Exception::new(type_),
            e => models::Exception {
                type_: "https://datatracker.ietf.org/doc/rfc7807/".to_string(), // TODO: Application error URL
                title: None,
                detail: Some(e.to_string()),
                status: None,
                instance: None,
            },
        }
    }
}
