use crate::{user_permission::Target, Paging};
use broker::Subscriber;
use derive_model::Model;
use derive_new_model::NewModel;
use derive_query::Query;
use derive_update_model::UpdateModel;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use to_params::{FromParams, ToParams};
use util::{
    error::UtilError,
    make_sort,
    store::{NewModel, PaginatedResult, UpdateModel, RODB, RWDB},
    AppState, FromParams, ToParams,
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

#[derive(Debug, Serialize, Deserialize, ToSchema, Default, Clone, Query, FromParams, ToParams)]
pub struct Query {
    id: Option<Uuid>,
    display_name: Option<String>,
    email: Option<String>,
    #[serde(flatten)]
    sort: Option<UserReadModelSort>,
    #[serde(flatten)]
    paging: Option<Paging>,
}

impl Subscriber for UserReadModel {
    type MessageType = Query;

    fn handle_message(
        &self,
        message: Self::MessageType,
        state: Arc<impl AppState>,
    ) -> impl std::future::Future<Output = Result<(), UtilError>> + Send {
        log::trace!(
            "message received on {} attempting to materialize read model",
            self.topic()
        );
        UserReadModel::materialize(
            message,
            state.get_ro_store().clone(),
            state.get_rw_store().clone(),
        )
    }

    fn topic(&self) -> String {
        "MaterializeUserReadModel".to_string()
    }
    fn group_name(&self) -> String {
        "MaterializeUserReadModel".to_string()
    }
}

impl UserReadModel {
    async fn materialize(query: Query, ro_db: RODB, rw_db: RWDB) -> Result<(), UtilError> {
        let query_result = Self::query(
            query,
            Some(
                "select id,external_id,display_name,email,permissions from user_readmodels_v"
                    .to_owned(),
            ),
            &ro_db,
        )
        .await?;
        if let Some(read_model) = query_result.data.first() {
            let _ = Self::upsert(read_model.clone(), &rw_db).await;

            return Ok(());
        }
        Err(UtilError::RowCantMaterialize)
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
        UserReadModel::materialize(query.clone(), state.get_ro_store().clone(), state.get_rw_store().clone())
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

        UserReadModel::materialize(query, state.get_ro_store().clone(), state.get_rw_store().clone())
            .await
            .unwrap();
    }
}
