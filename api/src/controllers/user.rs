use crate::error::ApiError;
use axum::{
    debug_handler,
    extract::{Json, Query, State},
};
use model::{
    user_permission::{Query as UserPermissionQuery, UserPermission},
    user_readmodel::{Query as ReadModelQuery, UserReadModel},
    State as ModelState,
};
use std::sync::Arc;
use tracing::instrument;
use util::{
    store::{Model, PaginatedResult},
    AppState,
};

#[utoipa::path(
    get,
    path = "/users",
    responses(
            (status = 200, description = "Get all users", body = PaginatedResult<UserReadModel>)
        )
)]
#[instrument(skip(api_state))]
#[debug_handler]
pub async fn get_users(
    Query(query): Query<ReadModelQuery>,
    State(api_state): State<Arc<ModelState>>,
) -> Result<Json<PaginatedResult<UserReadModel>>, ApiError> {
    Ok(Json(
        UserReadModel::query(query, None, api_state.get_ro_store())
            .await
            .map_err(ApiError::from)?,
    ))
}

#[utoipa::path(
    get,
    path = "/users_permissions",
    responses(
            (status = 200, description = "Get User permissions", body = String)
        )
)]
#[instrument(skip(api_state))]
#[debug_handler]
pub async fn get_user_permissions(
    Query(query): Query<UserPermissionQuery>,
    State(api_state): State<Arc<ModelState>>,
) -> Result<Json<PaginatedResult<UserPermission>>, ApiError> {
    Ok(Json(
        UserPermission::query(query, None, api_state.get_ro_store())
            .await
            .map_err(ApiError::from)?,
    ))
}
