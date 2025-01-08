use crate::{
    controllers::JsonOrHtml,
    error::ApiError,
    extractors::content_type::{ContentType, ContentTypes},
    respond_with,
};
use axum::{
    debug_handler,
    extract::{Json, Path, Query, State},
    response::IntoResponse,
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
use uuid::Uuid;

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
    ContentType(content_type): ContentType,
    Query(query): Query<ReadModelQuery>,
    State(api_state): State<Arc<ModelState>>,
) -> Result<impl IntoResponse, ApiError> {
    let users = UserReadModel::query(query, None, api_state.get_ro_store())
        .await
        .map_err(ApiError::from)?;

    respond_with!(
        content_type,
        frontend::users::get_users(users, &api_state).await?,
        users
    )
}

#[utoipa::path(
    get,
    path = "/users/:id",
    responses(
            (status = 200, description = "Get user by id", body = UserReadModel)
        )
)]
#[instrument(skip(api_state))]
#[debug_handler]
pub async fn get_user(
    ContentType(content_type): ContentType,
    Path(id): Path<Uuid>,
    State(api_state): State<Arc<ModelState>>,
) -> Result<impl IntoResponse, ApiError> {
    let query = ReadModelQuery {
        id: Some(id),
        ..ReadModelQuery::default()
    };
    let user = UserReadModel::get(query, None, api_state.get_ro_store())
        .await
        .map_err(ApiError::from)?;

    respond_with!(
        content_type,
        frontend::users::get_user(user, &api_state).await?,
        user
    )
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
