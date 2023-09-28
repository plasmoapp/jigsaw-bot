use std::{collections::HashMap, sync::Arc};

use axum::extract::ws::Message;
use jigsaw_common::util::config::default_extract_config;
use redis::aio::MultiplexedConnection;
use shrinkwraprs::Shrinkwrap;
use tokio::sync::{
    broadcast::{self, Receiver, Sender},
    RwLock,
};
use uuid::Uuid;

use crate::config::Config;

use eyre::Report;

#[derive(Clone, Shrinkwrap)]
pub struct AppState {
    redis: MultiplexedConnection,
    #[shrinkwrap(main_field)]
    inner: Arc<AppStateInner>,
}

impl AppState {
    pub async fn new() -> Result<Self, Report> {
        let config = default_extract_config::<Config>()?;

        let redis = redis::Client::open(config.redis_url.as_str())?
            .get_multiplexed_tokio_connection()
            .await?;

        let inner = AppStateInner::new(config);

        let result = Self {
            inner: Arc::new(inner),
            redis,
        };

        Ok(result)
    }
}

pub struct AppStateInner {
    pub config: Config,
    pub telegram_web_secret: ring::hmac::Key,
    puzzle_uuid_to_sender: RwLock<HashMap<Uuid, Sender<Message>>>,
}

impl AppStateInner {
    pub fn new(config: Config) -> Self {
        let telegram_web_secret = config.get_telegram_web_secret();

        Self {
            config,
            telegram_web_secret,
            puzzle_uuid_to_sender: RwLock::new(HashMap::new()),
        }
    }

    pub async fn get_channel(&self, puzzle_uuid: &Uuid) -> (Receiver<Message>, Sender<Message>) {
        let lock = self.puzzle_uuid_to_sender.read().await;
        match lock.get(puzzle_uuid) {
            Some(sender) => (sender.subscribe(), sender.clone()),
            None => {
                drop(lock);
                let mut lock = self.puzzle_uuid_to_sender.write().await;
                let (sender, _) = broadcast::channel(100);
                lock.insert(*puzzle_uuid, sender.clone());
                (sender.subscribe(), sender.clone())
            }
        }
    }

    pub async fn get_sender(&self, puzzle_uuid: &Uuid) -> Option<Sender<Message>> {
        let lock = self.puzzle_uuid_to_sender.read().await;
        lock.get(puzzle_uuid).cloned()
    }

    pub async fn remove_if_no_receivers(&self, puzzle_uuid: &Uuid) -> Option<bool> {
        let lock = self.puzzle_uuid_to_sender.read().await;
        if lock.get(puzzle_uuid)?.receiver_count() != 0 {
            return Some(false);
        }
        drop(lock);
        let mut lock = self.puzzle_uuid_to_sender.write().await;
        lock.remove(puzzle_uuid)?;
        Some(true)
    }
}
