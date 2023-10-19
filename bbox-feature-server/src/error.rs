//! Error and Result types.
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Geometry format error")]
    GeometryFormatError,
    #[error("datasource setup error - {0}")]
    DatasourceSetupError(String),
    #[error("datasource `{0}` not found")]
    DatasourceNotFound(String),
    // Database errors
    #[error(transparent)]
    DbError(#[from] sqlx::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
