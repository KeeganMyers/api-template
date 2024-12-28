use tokio::task::JoinHandle;
use util::{env::Env, error::UtilError, AppState};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub enum MessageBody {
    R(Uuid),
    S(String),
    B(Vec<u8>),
}

#[derive(Clone, Debug)]
pub struct Message {
    pub body: Option<MessageBody>,
    pub should_stop: bool,
}

#[allow(async_fn_in_trait)]
pub trait Subscriber: Send + Sync {
    async fn handle_message(
        &self,
        message: &Message,
        topic: &str,
        app_state: &impl AppState,
    ) -> Result<(), UtilError>;
    fn topic(&self) -> String;
}

#[allow(async_fn_in_trait)]
pub trait BrokerLayer {
    type BrokerConnection;
    async fn new(env: &Env) -> Result<Self::BrokerConnection, UtilError>;
    async fn publish(&self, topic: &str, message: &Message) -> Result<(), UtilError>;
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
