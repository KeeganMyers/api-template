use chrono::{DateTime, Utc};
use derive_model::Model;
use derive_new_model::NewModel;
use derive_update_model::UpdateModel;
use serde::{Deserialize, Serialize};
use util::{
    error::UtilError,
    store::{NewModel, PaginatedResult, UpdateModel, RODB, RWDB},
};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, sqlx::Type, Default, PartialEq, Clone)]
#[sqlx(type_name = "target", rename_all = "lowercase")]
pub enum Target {
    #[default]
    User,
    UserPermission,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Default, Clone, Model, ToSchema)]
#[model(table_name = "user_permissions")]
pub struct UserPermission {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub user_id: Uuid,
    pub target: Target,
    pub create_record: bool,
    pub update_record: bool,
    pub view_record: bool,
    pub delete_record: bool,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Default, Clone, NewModel, ToSchema)]
pub struct NewUserPermission {
    pub id: Uuid,
    pub user_id: Uuid,
    pub target: Target,
    pub create_record: bool,
    pub update_record: bool,
    pub view_record: bool,
    pub delete_record: bool,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Default, Clone, UpdateModel, ToSchema)]
pub struct UpdateUserPermission {
    pub id: Uuid,
    pub target: Option<Target>,
    pub create_record: Option<bool>,
    pub update_record: Option<bool>,
    pub view_record: Option<bool>,
    pub delete_record: Option<bool>,
}
