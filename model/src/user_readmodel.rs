use crate::{error::ModelError, user_permission::Target, Paging};
use derive_model::Model;
use derive_new_model::NewModel;
use derive_query::Query;
use derive_update_model::UpdateModel;
use serde::{Deserialize, Serialize};
use util::{
    error::UtilError,
    make_sort,
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
    pub user_id: Uuid,
}

#[derive(
    Debug, Serialize, Deserialize, sqlx::FromRow, Default, Clone, Model, NewModel, UpdateModel,
)]
#[model(table_name = "user_readmodels")]
pub struct UserReadModel {
    pub id: Uuid,
    pub external_id: Option<String>,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub permissions: Option<serde_json::Value>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, ToSchema, Default, Clone)]
#[serde(rename_all = "snake_case")]
pub enum SortColumn {
    #[default]
    DisplayName,
    Email,
}

make_sort!(UserReadModelSort, SortColumn);

#[derive(Debug, Serialize, Deserialize, ToSchema, Default, Clone, Query)]
pub struct Query {
    id: Option<Uuid>,
    display_name: Option<String>,
    email: Option<String>,
    #[serde(flatten)]
    sort: Option<UserReadModelSort>,
    #[serde(flatten)]
    paging: Option<Paging>,
}

impl UserReadModel {
    async fn materialize(query: Query, ro_db: &RODB, rw_db: &RWDB) -> Result<Self, ModelError> {
        let query_result = Self::query(
            query,
            Some(
                "select id,external_id,display_name,email,permissions from user_readmodels_v"
                    .to_owned(),
            ),
            ro_db,
        )
        .await?;
        if let Some(read_model) = query_result.data.first() {
            return Self::upsert(read_model.clone(), rw_db)
                .await
                .map_err(ModelError::from);
        }
        Err(ModelError::RowCantMaterialize)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        user::{NewUser, User},
        user_permission::{NewUserPermission, Target, UserPermission},
    };
    use util::{tests::TestApiState, AppState};

    #[tokio::test]
    async fn materialize_user_read_model() {
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
        UserReadModel::materialize(query.clone(), state.get_ro_store(), state.get_rw_store())
            .await
            .unwrap();

        let new_perm = NewUserPermission {
            user_id: user.id,
            target: Target::User,
            create_record: true,
            ..NewUserPermission::default()
        };
        UserPermission::insert(new_perm, state.get_rw_store())
            .await
            .unwrap();

        UserReadModel::materialize(query, state.get_ro_store(), state.get_rw_store())
            .await
            .unwrap();
    }
}
