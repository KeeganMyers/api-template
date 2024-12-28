use crate::{error::ApiError, ApiState};
use async_trait::async_trait;
use axum_core::extract::{FromRef, FromRequestParts};
use casdoor_rust_sdk::{AuthService, CasdoorConfig, CasdoorUser};
use http::{header::AUTHORIZATION, request::Parts, StatusCode};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use tracing::instrument;
use util::AppState;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AuthUserHeaderCustom(pub String);

#[derive(Deserialize, Serialize, Default)]
pub struct AuthUser(pub CasdoorUser);

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    ApiState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ApiError;

    #[instrument(skip_all)]
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<AuthUser, Self::Rejection> {
        let app_state = ApiState::from_ref(state);
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
            let user = auth_service
                .parse_jwt_token(token.0)
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
