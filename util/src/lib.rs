pub mod env;
pub mod error;
pub mod macros;
pub mod store;
use utoipa::ToSchema;

use async_trait::async_trait;
use env::{Env, PostgresConfig};
use serde::{Deserialize, Serialize};
use store::{RODB, RWDB};

pub trait AppConfig {
    fn get_rw_store_settings(&self) -> &PostgresConfig;
    fn get_ro_store_settings(&self) -> &PostgresConfig;
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(untagged)]
pub enum JsonNum {
    S(String),
    I(i64),
    U(u64),
}

#[async_trait]
pub trait AppState {
    type StateType;
    type ErrorType;

    async fn from_env(env: Env) -> Result<Self::StateType, Self::ErrorType>;
    fn get_rw_store(&self) -> &RWDB;
    fn get_ro_store(&self) -> &RODB;
    fn get_env(&self) -> &Env;
}
