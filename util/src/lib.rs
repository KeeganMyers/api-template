pub mod env;
pub mod error;
pub mod store;

use env::{Env, PostgresConfig};
use store::{RODB, RWDB};
use async_trait::async_trait;

pub trait AppConfig {
    fn get_rw_store_settings(&self) -> &PostgresConfig;
    fn get_ro_store_settings(&self) -> &PostgresConfig;
}

#[async_trait]
pub trait AppState {
    type StateType;
    type ErrorType;

    async fn from_env(env: Env) -> Result<Self::StateType,Self::ErrorType>;
    fn get_rw_store(&self) -> &RWDB;
    fn get_ro_store(&self) -> &RODB;
    fn get_env(&self) -> &Env;
}
