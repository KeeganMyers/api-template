use crate::{error::ApiError, ApiState};
use axum::{debug_handler, extract::State, Json};
use casdoor_rust_sdk::{AuthService, CasdoorConfig};
use casdoor_rust_sdk::{CasdoorUser, UserService};
use std::sync::Arc;
use util::AppState;

#[utoipa::path(
    post,
    path = "/auth_login",
    responses(
            (status = 200, description = "Start Auth SSO flow by re-directing to Casdoor", body = String)
        )
)]
#[debug_handler]
pub async fn auth_login(State(api_state): State<Arc<ApiState>>) -> Result<Json<String>, ApiError> {
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
pub async fn auth_signup(State(api_state): State<Arc<ApiState>>) -> Result<Json<String>, ApiError> {
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

#[utoipa::path(
    post,
    path = "/auth_callback",
    params(
        ("code" = String, Path, description = "auth code returned by auth service")
    ),
)]
#[debug_handler]
pub async fn auth_callback(
    State(api_state): State<Arc<ApiState>>,
    code: String,
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
        let auth_service = AuthService::new(&config);
        let token = auth_service.get_auth_token(code).map_err(ApiError::from)?;
        let user = auth_service
            .parse_jwt_token(token)
            .map_err(ApiError::from)?;
        return Ok(Json(user));
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
    State(api_state): State<Arc<ApiState>>,
    name: String,
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
        let user = user_service.get_user(name).await.unwrap();
        return Ok(Json(user));
    }

    Err(ApiError::AuthConfigNotConfigured)
}

#[utoipa::path(get, path = "/auth_users")]
#[debug_handler]
pub async fn get_auth_users(
    State(api_state): State<Arc<ApiState>>,
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
    State(api_state): State<Arc<ApiState>>,
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
    State(api_state): State<Arc<ApiState>>,
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
