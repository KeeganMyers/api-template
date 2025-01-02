use super::{error::ModelError, Paging, Role};
use casdoor_rust_sdk::CasdoorUser;
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

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Default, Clone, Model)]
#[model(table_name = "users")]
pub struct User {
    pub id: Uuid,
    pub external_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub role: Role,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Default, Clone, NewModel)]
pub struct NewUser {
    pub role: Role,
    pub external_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Default, Clone, UpdateModel)]
pub struct UpdateUser {
    pub last_login: Option<DateTime<Utc>>,
    pub role: Option<Role>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, ToSchema, Default, Clone)]
pub enum SortColumn {
    #[default]
    CreatedAt,
    UpdatedAt,
    DisplayName,
}

make_sort!(UserSort, SortColumn);

#[derive(Debug, Serialize, Deserialize, ToSchema, Default, Clone, Query)]
pub struct Query {
    display_name: Option<String>,
    email: Option<String>,
    #[serde(flatten)]
    sort: Option<UserSort>,
    #[serde(flatten)]
    paging: Option<Paging>,
}

impl User {
    pub async fn create(new_user: NewUser, db: RWDB) -> Result<User, ModelError> {
        sqlx::query_as!(
            Self,
            r#"
            INSERT INTO users (external_id,role,display_name,email)
            values ($1,$2,$3,$4)
            RETURNING id,external_id,created_at,updated_at,last_login,role as "role!: Role",display_name,email
            "#,
            new_user.external_id,
            new_user.role as Role,
            new_user.display_name,
            new_user.email
        )
        .fetch_one(db.get_conn())
        .await
        .map_err(|e| ModelError::from(UtilError::from(e)))
    }

    pub async fn get(id: Uuid, db: RODB) -> Result<User, ModelError> {
        sqlx::query_as!(
            Self,
            r#"
            SELECT id,external_id,created_at,updated_at,last_login,role as "role!: Role",display_name,email
            FROM users
            where id = $1
            "#,
            id
        )
        .fetch_one(db.get_conn())
        .await
        .map_err(|e| ModelError::from(UtilError::from(e)))
    }

    pub async fn get_paginated(
        query: Query,
        db: &RODB,
    ) -> Result<PaginatedResult<Self>, ModelError> {
        Self::query(query, None, db).await.map_err(ModelError::from)
    }

    pub async fn get_by_auth_user(c_user: CasdoorUser, db: &RODB) -> Result<User, ModelError> {
        sqlx::query_as!(
            Self,
            r#"
            SELECT id,external_id,created_at,updated_at,last_login,role as "role!: Role",display_name,email
            FROM users
            where external_id = $1
            "#,
            c_user.id
        )
        .fetch_one(db.get_conn())
        .await
        .map_err(|e| ModelError::from(UtilError::from(e)))
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
        let updated_model = UpdateUser {
            display_name: Some("changed name".to_string()),
            email: Some("someone@somewhere.com".to_string()),
            ..UpdateUser::default()
        };

        //async fn update(id: Uuid,updated_model: impl UpdateModel,db: &RWDB) -> Result<Self,UtilError>;
        User::update(user.id, updated_model, state.get_rw_store())
            .await
            .unwrap();
    }
}
