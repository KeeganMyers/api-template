use deadpool_redis::redis::cmd;
use tokio::task::JoinHandle;
use util::store::{CacheLayer, ConnectionPool, Redis};
use util::{env::Env, error::UtilError, macros::redis_op, AppState};
use util::{FromParams, ToParams};
use uuid::Uuid;

#[allow(async_fn_in_trait)]
pub trait Subscriber: Send + Sync {
    async fn handle_message(
        &self,
        message: &(impl FromParams + ToParams),
        topic: &str,
        app_state: &impl AppState,
    ) -> Result<(), UtilError>;
    fn topic() -> String;
    fn group_name() -> String;
}

#[allow(async_fn_in_trait)]
pub trait BrokerLayer: Sized {
    async fn new(env: &Env) -> Result<Self, UtilError>;
    async fn publish(
        &self,
        topic: &str,
        message: &(impl FromParams + ToParams),
    ) -> Result<(), UtilError>;
    async fn subscribe(
        &self,
        topic: &str,
        subscriber: impl Subscriber + 'static,
        app_state: &impl AppState,
    ) -> Result<JoinHandle<()>, UtilError>;
    async fn topic_exists(&self, topic: &str) -> Result<bool, UtilError>;
    async fn add_topic(&self, _topic: &str) -> Result<(), UtilError> {
        Ok(())
    }
    async fn remove_queue(&self, topic: &str) -> Result<(), UtilError>;
}

#[derive(Clone)]
pub struct RedisStream {
    pub client: Redis,
    pub max_len: i64,
}

#[allow(async_fn_in_trait)]
impl BrokerLayer for RedisStream {
    async fn new(env: &Env) -> Result<Self, UtilError> {
        if let Some(redis) = &env.redis {
            return Ok(Self {
                client: Redis::new(&env).await?,
                max_len: redis
                    .stream_len
                    .as_ref()
                    .and_then(|l| l.parse::<i64>().ok())
                    .unwrap_or(1000),
            });
        }

        Err(UtilError::RedisNotConfigured)
    }

    async fn publish(
        &self,
        topic: &str,
        message: &(impl FromParams + ToParams),
    ) -> Result<(), UtilError> {
        let mut xargs: Vec<String> = vec![
            topic.into(),
            " MAXLEN ~ ".into(),
            self.max_len.to_string(),
            " * ".into(),
        ];
        let mut params = message.to_params();
        xargs.append(&mut params);
        let client = &self.client;
        redis_op!(client, cmd("XADD").arg(&xargs), String);
        Ok(())
    }

    async fn subscribe(
        &self,
        topic: &str,
        subscriber: impl Subscriber + 'static,
        app_state: &impl AppState,
    ) -> Result<JoinHandle<()>, UtilError> {
        unimplemented!()
    }

    ///Not required for redis streams but valuable for other message brokers
    async fn topic_exists(&self, topic: &str) -> Result<bool, UtilError> {
        Ok(true)
    }

    ///Not required for redis streams but valuable for other message brokers
    async fn add_topic(&self, _topic: &str) -> Result<(), UtilError> {
        Ok(())
    }

    ///Not required for redis streams but valuable for other message brokers
    async fn remove_queue(&self, topic: &str) -> Result<(), UtilError> {
        Ok(())
    }
}
