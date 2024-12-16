use crate::{error::UtilError, AppConfig};
use dotenv::dotenv;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct PostgresConfig {
    #[serde(rename = "pg_port")]
    pub port: String,
    #[serde(rename = "pg_user")]
    pub username: String,
    #[serde(rename = "pg_ro_user")]
    pub ro_username: String,
    #[serde(rename = "pg_db")]
    pub db_name: String,
    #[serde(rename = "pg_password")]
    pub password: String,
    #[serde(rename = "pg_host")]
    pub host: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Env {
    #[serde(flatten)]
    pub postgres: PostgresConfig,
    pub server_port: Option<u16>,
}

impl Env {
    pub fn from_env() -> Result<Self, UtilError> {
        dotenv().ok();
        envy::from_env::<Self>().map_err(UtilError::from)
    }
}

impl AppConfig for Env {
    fn get_rw_store_settings(&self) -> &PostgresConfig {
        &self.postgres
    }
    fn get_ro_store_settings(&self) -> &PostgresConfig {
        &self.postgres
    }
}
