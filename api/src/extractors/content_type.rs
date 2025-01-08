use crate::error::ApiError;
use async_trait::async_trait;
use axum_core::extract::{FromRef, FromRequestParts};
#[allow(unused_imports)]
use base64::engine::Engine;
#[allow(unused_imports)]
use casdoor_rust_sdk::{AuthService, CasdoorConfig, CasdoorUser};
use http::{
    header::{ACCEPT, CONTENT_TYPE},
    request::Parts,
};
use model::State as ModelState;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::sync::Arc;
use tracing::instrument;
#[allow(unused_imports)]
use util::{AppState, B64_ENGINE};

#[derive(Deserialize, Serialize, Default)]
pub struct ContentType(pub ContentTypes);
#[derive(Deserialize, Serialize, Default, Debug)]
pub enum ContentTypes {
    #[default]
    Json,
    Html,
    Other,
}

#[async_trait]
impl<S> FromRequestParts<S> for ContentType
where
    Arc<ModelState>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ApiError;

    #[instrument(skip_all)]
    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<ContentType, Self::Rejection> {
        Ok(Self::decode_request_parts(parts))
    }
}

impl ContentType {
    fn decode_request_parts(req: &mut Parts) -> Self {
        println!("req headers {:?}", req.headers);
        let content_type = req
            .headers
            .iter()
            .find(|(key, _value)| **key == CONTENT_TYPE || **key == ACCEPT)
            .and_then(|(_key, value)| value.to_str().ok());

        println!("got content type {:?}", content_type);
        match content_type {
            Some(c) if c.contains("application/json") => Self(ContentTypes::Json),
            Some(c) if c.contains("text/html") => Self(ContentTypes::Html),
            _ => Self(ContentTypes::Other),
        }
    }
}
