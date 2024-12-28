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
use std::{
    net::{SocketAddr, TcpListener},
    sync::Arc,
};
use tracing::instrument;
use util::{
    env::Env,
    store::{RODB, RWDB},
    AppState,
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[instrument]
async fn healthcheck() -> (StatusCode, &'static str) {
    (StatusCode::OK, "OK")
}

pub(crate) fn routes(app_state: Arc<ApiState>) -> Router {
    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/healthcheck", get(healthcheck))
        .route("/auth_login", post(auth::auth_login))
        .route("/auth_signup", post(auth::auth_signup))
        .route("/auth_callback", post(auth::auth_callback))
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

#[derive(Clone)]
pub struct ApiState {
    pub rw_db: RWDB,
    pub ro_db: RODB,
    pub env: Env,
}

#[derive(OpenApi)]
#[openapi(paths(), components(schemas()))]
pub struct ApiDoc;

impl AppState for ApiState {
    type StateType = ApiState;
    type ErrorType = ApiError;

    async fn from_env(env: Env) -> Result<Self::StateType, Self::ErrorType> {
        Ok(Self {
            rw_db: RWDB::connect(&env).await?,
            ro_db: RODB::connect(&env).await?,
            env,
        })
    }

    fn get_rw_store(&self) -> &RWDB {
        &self.rw_db
    }
    fn get_ro_store(&self) -> &RODB {
        &self.ro_db
    }
    fn get_env(&self) -> &Env {
        &self.env
    }
}

pub async fn start_server(app_state: ApiState) -> Result<(), ApiError> {
    let release = env!("CARGO_PKG_VERSION");
    let app = routes(Arc::new(app_state.clone())).into_make_service();
    let port = app_state.env.server_port.unwrap_or(8080);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let socket_http = TcpListener::bind(addr)?;
    let server_http = axum_server::from_tcp(socket_http).serve(app.clone());
    info!("starting server version {release} on port {port}");
    server_http.await.map_err(ApiError::from)
}
