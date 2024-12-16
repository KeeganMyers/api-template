use thiserror::Error as ThisError;
use util::error::UtilError;

#[derive(Debug, ThisError)]
pub enum ModelError {
    #[error(transparent)]
    Util(#[from] UtilError),
}
