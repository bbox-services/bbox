//! Error and Result types.
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("No node found")]
    NodeNotFound,
    #[error("No route found")]
    NoRouteFound,
    // Requests
    #[error("Argument error `{0}`")]
    ArgumentError(String),
    // General
    #[error("I/O error")]
    IoError(#[from] std::io::Error),
    #[error("Bincode error")]
    BincodeError(#[from] bincode::Error),
    #[error("Database error")]
    DbError(#[from] sqlx::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
