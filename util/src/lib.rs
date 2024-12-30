pub mod env;
pub mod error;
pub mod macros;
pub mod store;
use crate::store::CacheLayer;
use utoipa::ToSchema;

use base64::{
    alphabet,
    engine::{self, general_purpose},
};
use env::{Env, PostgresConfig};
use serde::{Deserialize, Serialize};
use store::{RODB, RWDB};

pub const B64_ENGINE: engine::GeneralPurpose =
    engine::GeneralPurpose::new(&alphabet::URL_SAFE, general_purpose::NO_PAD);

#[cfg(any(test, debug_assertions))]
pub mod tests;

pub trait AppConfig {
    fn get_rw_store_settings(&self) -> &PostgresConfig;
    fn get_ro_store_settings(&self) -> &PostgresConfig;
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, ToSchema)]
#[serde(untagged)]
pub enum JsonNum {
    S(String),
    I(i64),
    U(u64),
}

#[allow(async_fn_in_trait)]
pub trait AppState {
    type StateType;
    type ErrorType;

    async fn from_env(env: Env) -> Result<Self::StateType, Self::ErrorType>;
    fn get_rw_store(&self) -> &RWDB;
    fn get_ro_store(&self) -> &RODB;
    fn get_env(&self) -> &Env;
    fn cache(&self) -> Option<&impl CacheLayer>;
}

pub trait ToParams {
    fn to_params(&self) -> Vec<String>;
}

pub trait FromParams: Default {
    fn from_params(params: Vec<String>) -> Self;
}
