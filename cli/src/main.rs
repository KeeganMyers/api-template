use api::start_server;
use broker::{BrokerLayer, RedisStream};
use chrono::Utc;
use clap::Parser;
use dotenv::dotenv;
use futures::future::join_all;
use model::{subscribers, State as ModelState};
use std::error::Error;
use std::fs::File;
use std::sync::Arc;
use util::store::RWDB;
use util::{env::Env, AppState};

const CRATE_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(clap::Args, Clone, Debug)]
pub struct AddSqlMigration {
    /// Name of the new migration to be created,this will be appended to a versioning string
    /// so the name its self should not include a version or timestamp
    name: String,
}

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub enum Command {
    /// Start the api
    Api,
    /// Run all sql db migrations
    MigrateSql,
    /// Returns current crate version
    Version,
    /// Create a new sql migration file
    AddSqlMigration(AddSqlMigration),
    Broker,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    //set env vars if .env file is present
    dotenv().ok();
    match Command::parse() {
        Command::Api => {
            let env = Env::from_env()?;
            env_logger::init();
            let app_state = ModelState::from_env(env).await?;
            let server_handle = start_server(app_state);
            let _ = server_handle.await;
        }
        Command::MigrateSql => {
            let env = Env::from_env()?;
            env_logger::init();
            RWDB::migrate(&env).await?
        }
        Command::AddSqlMigration(details) => {
            let filename = format!(
                "migrations/sql/{}_{}.sql",
                Utc::now().format("%s"),
                details.name
            );
            File::create(&filename)?;
            println!("created migration {filename}");
        }
        Command::Version => println!("{}", CRATE_VERSION),
        Command::Broker => {
            let env = Env::from_env()?;
            env_logger::init();
            let broker = RedisStream::new(&env).await?;
            let app_state = Arc::new(ModelState::from_env(env.clone()).await?);
            let subs = subscribers();
            let subscriptions = broker.start_subscriptions(subs, &env, app_state).await?;
            if !subscriptions.is_empty() {
                join_all(subscriptions).await;
            }
        }
    }
    Ok(())
}
