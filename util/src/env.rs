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
pub struct Auth {
    #[serde(rename = "auth_endpoint")]
    pub endpoint: String,
    #[serde(rename = "auth_client_id")]
    pub client_id: String,
    #[serde(rename = "auth_client_secret")]
    pub client_secret: String,
    #[serde(rename = "auth_cert")]
    pub certificate: String,
    #[serde(rename = "auth_org_name")]
    pub org_name: String,
    #[serde(rename = "auth_app_name")]
    pub app_name: Option<String>,
    #[serde(rename = "redirect_url")]
    pub redirect_url: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Env {
    #[serde(flatten)]
    pub postgres: PostgresConfig,
    pub server_port: Option<u16>,
    #[serde(flatten)]
    pub auth: Option<Auth>,
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
