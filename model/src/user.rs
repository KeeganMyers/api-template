use super::{error::ModelError, Paging};
use chrono::{DateTime, Utc};
use derive_model::Model;
use derive_new_model::NewModel;
use derive_query::Query;
use derive_update_model::UpdateModel;
use serde::{Deserialize, Serialize};
use util::{
    error::UtilError,
    macros::make_sort,
    store::{NewModel, PaginatedResult, UpdateModel, RODB, RWDB},
};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Default, Clone, Model, ToSchema)]
#[model(table_name = "users")]
pub struct User {
    pub id: Uuid,
    pub external_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Default, Clone, NewModel, ToSchema)]
pub struct NewUser {
    pub external_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Default, Clone, UpdateModel, ToSchema)]
pub struct UpdateUser {
    pub last_login: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, ToSchema, Default, Clone)]
#[serde(rename_all = "snake_case")]
pub enum SortColumn {
    #[default]
    CreatedAt,
    UpdatedAt,
    DisplayName,
}

make_sort!(UserSort, SortColumn);

#[derive(Debug, Serialize, Deserialize, ToSchema, Default, Clone, Query)]
pub struct Query {
    pub id: Option<Uuid>,
    pub display_name: Option<String>,
    pub email: Option<String>,
    #[serde(flatten)]
    pub sort: Option<UserSort>,
    #[serde(flatten)]
    pub paging: Option<Paging>,
}

impl User {
    pub async fn get_paginated(
        query: Query,
        db: &RODB,
    ) -> Result<PaginatedResult<Self>, ModelError> {
        Self::query(query, None, db).await.map_err(ModelError::from)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use util::{tests::TestApiState, AppState};

    #[tokio::test]
    async fn create_user() {
        let state = TestApiState::from_test_env().await.unwrap();
        let new_user = NewUser {
            display_name: Some("Test User".to_string()),
            ..NewUser::default()
        };
        User::insert(new_user, state.get_rw_store()).await.unwrap();
    }

    #[tokio::test]
    async fn update_user() {
        let state = TestApiState::from_test_env().await.unwrap();
        let new_user = NewUser {
            display_name: Some("Test User".to_string()),
            ..NewUser::default()
        };
        let user = User::insert(new_user, state.get_rw_store()).await.unwrap();
        let query = Query {
            id: Some(user.id),
            ..Query::default()
        };

        let updated_model = UpdateUser {
            display_name: Some("changed name".to_string()),
            email: Some("someone@somewhere.com".to_string()),
            ..UpdateUser::default()
        };

        User::update(&query, updated_model, state.get_rw_store())
            .await
            .unwrap();
    }
}
