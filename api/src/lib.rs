mod error;
mod middleware;
mod extractors;

use crate::error::ApiError;
use async_trait::async_trait;
use axum::{
    http::StatusCode,
    routing::{get},
    Router,
};
use log::{info};
use std::{
    net::{SocketAddr, TcpListener},
    sync::Arc,
};
use util::{
    env::Env,
    store::{RODB, RWDB},
    AppState,
};

async fn healthcheck() -> (StatusCode, &'static str) {
    (StatusCode::OK, "OK")
}

pub(crate) fn routes(app_state: Arc<ApiState>) -> Router {
    Router::new()
        .route("/healthcheck", get(healthcheck))
        //.layer(from_fn_with_state(app_state.clone(),cache_request))
        .with_state(app_state)
}

#[derive(Clone)]
pub struct ApiState {
    pub rw_db: RWDB,
    pub ro_db: RODB,
    pub env: Env,
}

#[async_trait]
impl AppState for ApiState {
    type StateType = ApiState;
    type ErrorType = ApiError;

    async fn from_env(env: Env) -> Result<Self::StateType,Self::ErrorType> {
        Ok(Self {
            rw_db: RWDB::connect(&env).await?,
            ro_db: RODB::connect(&env).await?,
            env
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
