use derive_axum_errors::ErrorResponse;
use http::StatusCode;
use model::error::ModelError;
use thiserror::Error as ThisError;
use util::error::UtilError;

#[derive(ThisError, ErrorResponse)]
pub enum ApiError {
    #[error("Auth config not set auth operations cannot be run")]
    AuthConfigNotConfigured,
    #[error("{0}")]
    #[status(StatusCode::UNAUTHORIZED)]
    Auth(String),
    #[error(transparent)]
    StandardError(#[from] Box<dyn std::error::Error>),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    Model(#[from] ModelError),
    #[error(transparent)]
    JoinHandle(#[from] tokio::task::JoinError),
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
    #[error(transparent)]
    #[status(StatusCode::UNAUTHORIZED)]
    B64(#[from] base64::DecodeError),
    #[error(transparent)]
    #[status(StatusCode::UNAUTHORIZED)]
    Utf8(#[from] std::string::FromUtf8Error),
}
