use crate::{error::ApiError, extractors::auth_user::AuthUser};
use axum::{
    debug_handler,
    extract::{Path, Query, State},
    Json,
};
use base64::engine::Engine;
use casdoor_rust_sdk::{AuthService, CasdoorConfig};
use casdoor_rust_sdk::{CasdoorUser, UserService};
use model::State as ModelState;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::task;
use util::{AppState, B64_ENGINE};

#[utoipa::path(
    post,
    path = "/auth_login",
    responses(
            (status = 200, description = "Start Auth SSO flow by re-directing to Casdoor", body = String)
        )
)]
#[debug_handler]
pub async fn auth_login(
    State(api_state): State<Arc<ModelState>>,
) -> Result<Json<String>, ApiError> {
    if let Some(auth) = api_state.get_env().auth.clone() {
        let config = CasdoorConfig::new(
            auth.endpoint,
            auth.client_id,
            auth.client_secret,
            auth.certificate,
            auth.org_name,
            auth.app_name,
        );
        let auth_service = AuthService::new(&config);
        let redirect_url = auth_service.get_signin_url(auth.redirect_url);
        return Ok(Json(redirect_url));
    }
    Err(ApiError::AuthConfigNotConfigured)
}

#[utoipa::path(
    post,
    path = "/auth_signup",
    responses(
            (status = 200, description = "Sign up to service with new SSO user", body = String)
        )
)]
#[debug_handler]
pub async fn auth_signup(
    State(api_state): State<Arc<ModelState>>,
) -> Result<Json<String>, ApiError> {
    if let Some(auth) = api_state.get_env().auth.clone() {
        let config = CasdoorConfig::new(
            auth.endpoint,
            auth.client_id,
            auth.client_secret,
            auth.certificate,
            auth.org_name,
            auth.app_name,
        );
        let auth_service = AuthService::new(&config);

        let redirect_url = auth_service.get_signup_url_enable_password();
        return Ok(Json(redirect_url));
    }
    Err(ApiError::AuthConfigNotConfigured)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CallbackQuery {
    pub code: String,
    pub state: String,
}

pub type Jwt = String;
#[utoipa::path(
    get,
    path = "/auth_callback",
    params(
        ("code" = String, Path, description = "auth code returned by auth service")
    ),
)]
#[debug_handler]
pub async fn auth_callback(
    State(api_state): State<Arc<ModelState>>,
    Query(query): Query<CallbackQuery>,
) -> Result<Json<Jwt>, ApiError> {
    if let Some(auth) = api_state.get_env().auth.clone() {
        let token_result = task::spawn_blocking(move || {
            let config = CasdoorConfig::new(
                auth.endpoint,
                auth.client_id,
                auth.client_secret,
                auth.certificate,
                auth.org_name,
                auth.app_name,
            );
            let auth_service = AuthService::new(&config);
            let token = auth_service.get_auth_token(query.code).map_err(|e| {
                let err_msg = e.to_string();
                log::error!("get_auth_token() error: {}", err_msg);
                err_msg
            })?;

            let _user = auth_service.parse_jwt_token(token.clone()).map_err(|e| {
                let err_msg = e.to_string();
                log::error!("parse_jwt_token() error: {}", err_msg);
                err_msg
            })?;
            Ok(B64_ENGINE.encode(token))
        })
        .await
        .map_err(ApiError::from)?;

        return match token_result {
            Ok(token) => Ok(Json(token)),
            Err(e) => Err(ApiError::Auth(e)),
        };
    }
    Err(ApiError::AuthConfigNotConfigured)
}

#[utoipa::path(
    get,
    path = "/auth_user",
    params(
        ("name" = String, Path, description = "name of auth user in sso service")
    ),
)]
#[debug_handler]
pub async fn get_auth_user(
    State(api_state): State<Arc<ModelState>>,
    Path(name): Path<String>,
    _auth_user: AuthUser,
) -> Result<Json<CasdoorUser>, ApiError> {
    if let Some(auth) = api_state.get_env().auth.clone() {
        let config = CasdoorConfig::new(
            auth.endpoint,
            auth.client_id,
            auth.client_secret,
            auth.certificate,
            auth.org_name,
            auth.app_name,
        );
        let user_service = UserService::new(&config);
        let user = user_service.get_user(name).await?;
        return Ok(Json(user));
    }

    Err(ApiError::AuthConfigNotConfigured)
}

#[utoipa::path(get, path = "/auth_users")]
#[debug_handler]
pub async fn get_auth_users(
    State(api_state): State<Arc<ModelState>>,
    _auth_user: AuthUser,
) -> Result<Json<Vec<CasdoorUser>>, ApiError> {
    if let Some(auth) = api_state.get_env().auth.clone() {
        let config = CasdoorConfig::new(
            auth.endpoint,
            auth.client_id,
            auth.client_secret,
            auth.certificate,
            auth.org_name,
            auth.app_name,
        );
        let user_service = UserService::new(&config);
        let users = user_service.get_users().await.unwrap();
        return Ok(Json(users));
    }
    Err(ApiError::AuthConfigNotConfigured)
}

#[utoipa::path(delete, path = "/auth_user")]
#[debug_handler]
pub async fn delete_auth_user(
    State(api_state): State<Arc<ModelState>>,
    _auth_user: AuthUser,
    user: Json<CasdoorUser>,
) -> Result<Json<u16>, ApiError> {
    if let Some(auth) = api_state.get_env().auth.clone() {
        let config = CasdoorConfig::new(
            auth.endpoint,
            auth.client_id,
            auth.client_secret,
            auth.certificate,
            auth.org_name,
            auth.app_name,
        );
        let user_service = UserService::new(&config);
        let code = user_service.delete_user(user.0).await.unwrap();
        return Ok(Json(code.as_u16()));
    }
    Err(ApiError::AuthConfigNotConfigured)
}

#[utoipa::path(post, path = "/auth_user")]
pub async fn add_auth_user(
    State(api_state): State<Arc<ModelState>>,
    _auth_user: AuthUser,
    user: Json<CasdoorUser>,
) -> Result<Json<u16>, ApiError> {
    if let Some(auth) = api_state.get_env().auth.clone() {
        let config = CasdoorConfig::new(
            auth.endpoint,
            auth.client_id,
            auth.client_secret,
            auth.certificate,
            auth.org_name,
            auth.app_name,
        );
        let user_service = UserService::new(&config);

        let code = user_service.add_user(user.0).await.unwrap();
        return Ok(Json(code.as_u16()));
    }
    Err(ApiError::AuthConfigNotConfigured)
}
