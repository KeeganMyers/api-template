use crate::{
    env::{Env, PostgresConfig},
    error::UtilError,
    store::{Redis, RODB, RWDB},
    AppState, CacheLayer,
};

fn get_db_config() -> PostgresConfig {
    PostgresConfig {
        port: "5436".to_string(),
        username: "rwusertest".to_string(),
        ro_username: "rouser".to_string(),
        password: "test".to_string(),
        host: "localhost".to_string(),
        db_name: "api_template".to_string(),
    }
}

fn get_test_env() -> Env {
    Env {
        postgres: get_db_config(),
        server_port: Some(3031),
        auth: None,
        redis: None,
        watch_topics: None,
    }
}

#[derive(Clone)]
pub struct TestApiState {
    pub rw_db: RWDB,
    pub ro_db: RODB,
    pub env: Env,
    pub cache: Redis,
}

impl AppState for TestApiState {
    type StateType = TestApiState;
    type ErrorType = UtilError;

    async fn from_env(env: Env) -> Result<Self::StateType, Self::ErrorType> {
        Ok(Self {
            rw_db: RWDB::connect(&env).await?,
            ro_db: RODB::connect(&env).await?,
            cache: Redis::new(&env).await?,
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

    fn cache(&self) -> Option<&impl CacheLayer> {
        Some(&self.cache)
    }
}

impl TestApiState {
    pub async fn from_test_env() -> Result<Self, UtilError> {
        Self::from_env(get_test_env()).await
    }
}
