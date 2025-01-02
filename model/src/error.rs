use thiserror::Error as ThisError;
use util::error::UtilError;

#[derive(Debug, ThisError)]
pub enum ModelError {
    #[error("Cant Materialize view no rows match query")]
    RowCantMaterialize,
    #[error(transparent)]
    Util(#[from] UtilError),
}
