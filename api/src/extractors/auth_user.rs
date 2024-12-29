use crate::{error::ApiError, ApiState};
use async_trait::async_trait;
use axum_core::extract::{FromRef, FromRequestParts};
#[allow(unused_imports)]
use base64::engine::Engine;
#[allow(unused_imports)]
use casdoor_rust_sdk::{AuthService, CasdoorConfig, CasdoorUser};
use http::{header::AUTHORIZATION, request::Parts, StatusCode};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::sync::Arc;
use tracing::instrument;
#[allow(unused_imports)]
use util::{AppState, B64_ENGINE};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AuthUserHeaderCustom(pub String);

#[derive(Deserialize, Serialize, Default)]
pub struct AuthUser(pub CasdoorUser);

/// When running tests mock the auth call to make test results more predictable
#[cfg(any(test, debug_assertions))]
#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    Arc<ApiState>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ApiError;

    #[instrument(skip_all)]
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<AuthUser, Self::Rejection> {
        let app_state = Arc::<ApiState>::from_ref(state);
        let token = AuthUserHeaderCustom::decode_request_parts(parts)?;
        if let Some(_auth) = app_state.get_env().auth.clone() {
            if token.0 == "valid" {
                return Ok(AuthUser(CasdoorUser::default()));
            } else {
                return Err(ApiError::Auth(
                    "Bearer token invalid or expired".to_string(),
                ));
            }
        }
        Err(ApiError::AuthConfigNotConfigured)
    }
}

#[cfg(not(any(test, debug_assertions)))]
#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    Arc<ApiState>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ApiError;

    #[instrument(skip_all)]
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<AuthUser, Self::Rejection> {
        let app_state = Arc::<ApiState>::from_ref(state);
        let token = AuthUserHeaderCustom::decode_request_parts(parts)?;
        if let Some(auth) = app_state.get_env().auth.clone() {
            let config = CasdoorConfig::new(
                auth.endpoint,
                auth.client_id,
                auth.client_secret,
                auth.certificate,
                auth.org_name,
                auth.app_name,
            );
            let auth_service = AuthService::new(&config);
            let decoded_token = String::from_utf8(B64_ENGINE.decode(token.0)?)?;
            let user = auth_service
                .parse_jwt_token(decoded_token)
                .map_err(ApiError::from)?;

            return Ok(AuthUser(user));
        }
        Err(ApiError::AuthConfigNotConfigured)
    }
}

impl AuthUserHeader for AuthUserHeaderCustom {
    const ERROR_CODE: StatusCode = StatusCode::BAD_REQUEST;
    const ERROR_OVERWRITE: Option<&'static str> = None;

    fn from_header(contents: &str) -> Self {
        Self(contents.to_string())
    }
}

pub trait AuthUserHeader: Sized {
    const ERROR_CODE: StatusCode;

    const ERROR_OVERWRITE: Option<&'static str>;
    fn from_header(contents: &str) -> Self;

    #[instrument(skip_all)]
    fn decode_request_parts(req: &mut Parts) -> Result<Self, ApiError> {
        let authorization = req
            .headers
            .iter()
            .find(|(key, _value)| **key == AUTHORIZATION)
            .map(|(_key, value)| value)
            .ok_or(ApiError::MissingAuth)?
            .to_str()
            .map_err(|_| ApiError::InvalidAuthHeaderChars)?;

        let split = authorization.split_once(' ');
        match (split, authorization) {
            (Some(("Bearer", contents)), _) => Ok(Self::from_header(contents)),
            _ => Err(ApiError::HeaderDecodeBearer),
        }
    }
}
