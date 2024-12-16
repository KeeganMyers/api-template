use crate::{env::PostgresConfig, error::UtilError, AppConfig};

pub trait RWStore {}
pub trait ROStore {}

#[derive(Clone)]
pub struct RWDB(PgPool);
#[derive(Clone)]
pub struct RODB(PgPool);

use sqlx::{postgres::PgPoolOptions, PgPool};

impl RODB {
    pub fn get_conn(&self) -> &PgPool {
        &self.0
    }

    pub fn connect_str(env: &PostgresConfig) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            env.ro_username, env.password, env.host, env.port, env.db_name
        )
    }

    pub async fn connect(state: &impl AppConfig) -> Result<Self, UtilError> {
        let connection = Self::connect_str(&state.get_rw_store_settings());
        let pool = PgPoolOptions::new()
            .min_connections(5)
            .connect(&connection)
            .await?;
        Ok(Self(pool))
    }
}

impl RWDB {
    pub fn get_conn(&self) -> &PgPool {
        &self.0
    }

    pub fn connect_str(env: &PostgresConfig) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            env.username, env.password, env.host, env.port, env.db_name
        )
    }

    pub async fn connect(state: &impl AppConfig) -> Result<Self, UtilError> {
        let connection = Self::connect_str(&state.get_rw_store_settings());
        let pool = PgPoolOptions::new()
            .min_connections(5)
            .connect(&connection)
            .await?;
        Ok(Self(pool))
    }

    pub async fn migrate(state: &impl AppConfig) -> Result<(), UtilError> {
        let pool = Self::connect(state).await?;
        sqlx::migrate!("../migrations/sql").run(&pool.0).await?;
        Ok(())
    }
}
