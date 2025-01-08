pub mod auth;
pub mod user;

use axum::{
    response::{Html, IntoResponse, Response},
    Json,
};
use http::StatusCode;
use serde::Serialize;

#[derive(Debug, Clone)]
pub enum JsonOrHtml<T> {
    Json(Json<T>),
    Html(Html<String>),
}

impl<T: Sized + Serialize> IntoResponse for JsonOrHtml<T> {
    fn into_response(self) -> Response {
        match self {
            Self::Json(json) => json.into_response(),
            Self::Html(html) => (StatusCode::OK, html).into_response(),
        }
    }
}
