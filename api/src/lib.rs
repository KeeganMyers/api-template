mod controllers;
mod error;
mod extractors;
mod middleware;

use crate::controllers::auth;
use crate::error::ApiError;
use axum::{
    http::StatusCode,
    routing::{get, post},
    Router,
};
use log::info;
use model::State as ModelState;
use std::{
    net::{SocketAddr, TcpListener},
    sync::Arc,
};
use tracing::instrument;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[instrument]
async fn healthcheck() -> (StatusCode, &'static str) {
    (StatusCode::OK, "OK")
}

pub(crate) fn routes(app_state: Arc<ModelState>) -> Router {
    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/healthcheck", get(healthcheck))
        .route("/auth_login", post(auth::auth_login))
        .route("/auth_signup", post(auth::auth_signup))
        .route("/auth_callback", get(auth::auth_callback))
        .route("/auth_users/:name", get(auth::get_auth_user))
        .route(
            "/auth_users",
            get(auth::get_auth_users)
                .delete(auth::delete_auth_user)
                .post(auth::add_auth_user),
        )
        //.layer(from_fn_with_state(app_state.clone(),cache_request))
        .with_state(app_state)
}

#[derive(OpenApi)]
#[openapi(
    paths(
        auth::auth_login,
        auth::auth_signup,
        auth::auth_callback,
        auth::get_auth_user,
        auth::get_auth_users,
        auth::delete_auth_user,
        auth::add_auth_user,
    ),
    components(schemas())
)]
pub struct ApiDoc;

pub async fn start_server(app_state: ModelState) -> Result<(), ApiError> {
    let release = env!("CARGO_PKG_VERSION");
    let app = routes(Arc::new(app_state.clone())).into_make_service();
    let port = app_state.env.server_port.unwrap_or(8080);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let socket_http = TcpListener::bind(addr)?;
    let server_http = axum_server::from_tcp(socket_http).serve(app.clone());
    info!("starting server version {release} on port {port}");
    server_http.await.map_err(ApiError::from)
}
