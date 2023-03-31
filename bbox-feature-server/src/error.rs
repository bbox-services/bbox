//! Error and Result types.
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Geometry format error")]
    GeometryFormatError,
    // General
    #[error("Database error")]
    DbError(#[from] sqlx::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
