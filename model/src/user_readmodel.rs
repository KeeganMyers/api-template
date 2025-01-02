use crate::user_permission::Target;
use derive_model::Model;
use serde::{Deserialize, Serialize};
use util::{
    error::UtilError,
    store::{NewModel, PaginatedResult, UpdateModel, RODB, RWDB},
};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Default, Clone, ToSchema)]
pub struct EmbeddedPermission {
    pub id: Uuid,
    pub target: Target,
    pub create_record: bool,
    pub update_record: bool,
    pub view_record: bool,
    pub delete_record: bool,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Default, Clone, Model)]
#[model(table_name = "user_readmodels")]
pub struct UserReadModel {
    pub id: Uuid,
    pub external_id: Option<String>,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub permissions: Vec<sqlx::types::Json<EmbeddedPermission>>,
}
