use std::{collections::HashMap, sync::Arc};

use crate::model::ws_message::WsMessage;
use tokio::sync::{
    broadcast::{self, Receiver, Sender},
    RwLock,
};
use uuid::Uuid;

pub struct WsState {
    puzzle_uuid_to_sender: RwLock<HashMap<Uuid, Sender<Arc<WsMessage>>>>,
}

impl WsState {
    pub async fn get_channel(
        &self,
        puzzle_uuid: &Uuid,
    ) -> (Receiver<Arc<WsMessage>>, Sender<Arc<WsMessage>>) {
        let lock = self.puzzle_uuid_to_sender.read().await;
        match lock.get(puzzle_uuid) {
            Some(sender) => (sender.subscribe(), sender.clone()),
            None => {
                drop(lock);
                let mut lock = self.puzzle_uuid_to_sender.write().await;
                let (sender, _) = broadcast::channel::<Arc<WsMessage>>(100);
                lock.insert(*puzzle_uuid, sender.clone());
                (sender.subscribe(), sender.clone())
            }
        }
    }

    pub async fn get_sender(&self, puzzle_uuid: &Uuid) -> Option<Sender<Arc<WsMessage>>> {
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

impl Default for WsState {
    fn default() -> Self {
        Self {
            puzzle_uuid_to_sender: RwLock::new(HashMap::new()),
        }
    }
}
