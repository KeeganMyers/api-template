use derive_axum_errors::ErrorResponse;
use http::StatusCode;
use model::error::ModelError;
use thiserror::Error as ThisError;
use util::error::UtilError;

#[derive(ThisError, ErrorResponse)]
pub enum ApiError {
    #[error("Auth config not set auth operations cannot be run")]
    AuthConfigNotConfigured,
    #[error(transparent)]
    StandardError(#[from] Box<dyn std::error::Error>),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    Model(#[from] ModelError),
    #[status(StatusCode::UNAUTHORIZED)]
    #[error("`Authorization` header  or Api Key is missing")]
    MissingAuth,
    #[status(StatusCode::UNAUTHORIZED)]
    #[error("`Authorization` header contains invalid characters")]
    InvalidAuthHeaderChars,
    #[status(StatusCode::UNAUTHORIZED)]
    #[error("`Authorization` header must be a bearer token")]
    HeaderDecodeBearer,
    #[error(transparent)]
    Util(#[from] UtilError),
}
