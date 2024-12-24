use derive_axum_errors::ErrorResponse;
use model::error::ModelError;
use thiserror::Error as ThisError;
use util::error::UtilError;

#[derive(ThisError, ErrorResponse)]
pub enum ApiError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    Model(#[from] ModelError),
    #[error(transparent)]
    Util(#[from] UtilError),
}
