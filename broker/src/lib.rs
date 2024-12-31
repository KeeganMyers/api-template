use deadpool_redis::redis::{cmd, Value};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use to_params::{FromParams, ToParams};
use tokio::task;
use tokio::task::JoinHandle;
use util::store::{CacheLayer, ConnectionPool, Redis};
use util::{env::Env, error::UtilError, macros::redis_op, AppState};
use util::{FromParams, ToParams};
use uuid::Uuid;

async fn redirect() -> Result<(), UtilError> {
    Ok(())
}

#[derive(Default)]
pub struct TestSubscriber {}
impl Subscriber for TestSubscriber {
    type MessageType = TestMessage;

    fn handle_message(
        &self,
        message: Self::MessageType,
        _app_state: Arc<impl AppState>,
    ) -> impl std::future::Future<Output = Result<(), UtilError>> + Send {
        println!("got message {:?}", message);
        redirect()
    }
    fn topic(&self) -> String {
        "TestStream".to_string()
    }
    fn group_name(&self) -> String {
        "TestGroup".to_string()
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, ToParams, FromParams)]
pub struct TestMessage {
    pub attr1: String,
}

pub fn subscribers() -> Vec<impl Subscriber> {
    let subs = vec![TestSubscriber::default()];
    subs
}

#[allow(async_fn_in_trait)]
pub trait Subscriber: Send + Sync {
    type MessageType: FromParams + ToParams;

    fn handle_message(
        &self,
        message: Self::MessageType,
        app_state: Arc<impl AppState>,
    ) -> impl std::future::Future<Output = Result<(), UtilError>> + Send;
    fn topic(&self) -> String;
    fn group_name(&self) -> String;
    fn parse_message(&self, message: &Value) -> Result<Self::MessageType, UtilError> {
        if let Some(params) = message
            .as_sequence()
            .and_then(|v| v.first())
            .and_then(|v| v.as_sequence())
            .and_then(|v| v.get(1))
            .and_then(|v| v.as_sequence())
            .and_then(|v| v.first())
            .and_then(|v| v.as_sequence())
            .and_then(|v| v.get(1))
            .and_then(|v| v.as_sequence())
            .map(|v| {
                v.iter()
                    .filter_map(|v| {
                        if let Value::BulkString(data) = v {
                            String::from_utf8(data.clone()).ok()
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<String>>()
            })
        {
            return Ok(Self::MessageType::from_params(params));
        }
        Err(UtilError::RedisStreamParams)
    }
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
        subscriber: impl Subscriber + 'static,
        app_state: Arc<impl AppState + Clone + std::marker::Sync + std::marker::Send + 'static>,
    ) -> Result<JoinHandle<Result<(), UtilError>>, UtilError>;
    async fn start_subscriptions(
        &self,
        env: &Env,
        app_state: Arc<impl AppState + Clone + std::marker::Sync + std::marker::Send + 'static>,
    ) -> Result<Vec<JoinHandle<Result<(), UtilError>>>, UtilError>;
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
                client: Redis::new(env).await?,
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
        let params = message.to_params();
        let client = &self.client;
        redis_op!(
            client,
            cmd("XADD")
                .arg(topic)
                .arg("MAXLEN")
                .arg("~")
                .arg(self.max_len)
                .arg("*")
                .arg(&params),
            String
        )?;
        Ok(())
    }

    async fn subscribe(
        &self,
        subscriber: impl Subscriber + 'static,
        app_state: Arc<impl AppState + Clone + std::marker::Sync + std::marker::Send + 'static>,
    ) -> Result<JoinHandle<Result<(), UtilError>>, UtilError> {
        let client = self.client.clone();
        let state = app_state.clone();
        let capture_topic = subscriber.topic();
        let join_handle: task::JoinHandle<_> = task::spawn(async move {
            let mkgroup = redis_op!(
                client,
                cmd("XGROUP")
                    .arg("CREATE")
                    .arg(capture_topic.clone())
                    .arg(subscriber.group_name())
                    .arg("$")
                    .arg("MKSTREAM"),
                String
            );

            if let Err(e) = &mkgroup {
                if !e.to_string().contains("Consumer Group name already exists") {
                    log::error!("Failed to create Redis group {:?}", mkgroup);
                }
            }

            loop {
                let mut message_processed = false;
                let message;

                let message_result = redis_op!(
                    client,
                    cmd("XREADGROUP")
                        .arg("GROUP")
                        .arg(subscriber.group_name())
                        .arg(Uuid::new_v4().to_string())
                        .arg("BLOCK")
                        .arg("0")
                        .arg("COUNT")
                        .arg("1")
                        .arg("STREAMS")
                        .arg(capture_topic.clone())
                        .arg(">")
                );

                if let Err(e) = message_result {
                    log::error!("Redis XREADGROUP error {:?}", e);
                    continue;
                } else {
                    message = message_result.unwrap();
                }

                let parsed_message = subscriber.parse_message(&message)?;
                if let Err(e) = subscriber
                    .handle_message(parsed_message, state.clone())
                    .await
                {
                    log::error!("Redis subscriber failed to handle message due to {:?}", e);
                } else {
                    message_processed = true
                }

                if message_processed {
                    if let Some(Value::BulkString(id_vec)) = message
                        .as_sequence()
                        .and_then(|v| v.first())
                        .and_then(|v| v.as_sequence())
                        .and_then(|v| v.get(1))
                        .and_then(|v| v.as_sequence())
                        .and_then(|v| v.first())
                        .and_then(|v| v.as_sequence())
                        .and_then(|v| v.first())
                    {
                        let id = String::from_utf8(id_vec.clone()).unwrap_or_default();

                        let ack_result: Result<Value, UtilError> = redis_op!(
                            client,
                            cmd("XACK")
                                .arg(capture_topic.clone())
                                .arg(subscriber.group_name())
                                .arg(id.clone())
                        );
                        if let Err(e) = ack_result {
                            log::error!(
                                "Redis xack error {:?} on stream {:?} id {:?}",
                                e,
                                capture_topic,
                                id
                            )
                        }
                    }
                }
            }
        });
        Ok(join_handle)
    }

    async fn start_subscriptions(
        &self,
        env: &Env,
        app_state: Arc<impl AppState + Clone + std::marker::Sync + std::marker::Send + 'static>,
    ) -> Result<Vec<JoinHandle<Result<(), UtilError>>>, UtilError> {
        let all_subscribers = subscribers();
        let watch_topics = env.watch_topics.clone().unwrap_or_default();
        let topic_names = watch_topics.split(",").collect::<HashSet<_>>();
        let mut subscribers = vec![];
        for subscriber in all_subscribers
            .into_iter()
            .filter(|s| topic_names.contains(s.topic().as_str()))
        {
            subscribers.push(self.subscribe(subscriber, app_state.clone()).await?)
        }
        Ok(subscribers)
    }

    ///Not required for redis streams but valuable for other message brokers
    async fn topic_exists(&self, _topic: &str) -> Result<bool, UtilError> {
        Ok(true)
    }

    ///Not required for redis streams but valuable for other message brokers
    async fn add_topic(&self, _topic: &str) -> Result<(), UtilError> {
        Ok(())
    }

    ///Not required for redis streams but valuable for other message brokers
    async fn remove_queue(&self, _topic: &str) -> Result<(), UtilError> {
        Ok(())
    }
}
