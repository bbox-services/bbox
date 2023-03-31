//! Error and Result types.
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    // Backend errors
    #[error(transparent)]
    BackendResponseError(#[from] reqwest::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
