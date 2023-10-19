//! Error and Result types.
use thiserror::Error;

#[derive(Error, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum Error {
    #[error("No node found")]
    NodeNotFound,
    #[error("No route found")]
    NoRouteFound,
    // Requests
    #[error("Argument error `{0}`")]
    ArgumentError(String),
    // General
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error("Bincode error")]
    BincodeError(#[from] bincode::Error),
    #[error(transparent)]
    DbError(#[from] sqlx::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
