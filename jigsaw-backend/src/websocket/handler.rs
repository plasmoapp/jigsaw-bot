use std::{collections::HashMap, str::FromStr};

use axum::extract::ws::{Message, WebSocket};
use futures::{stream::SplitSink, SinkExt, StreamExt};
use jigsaw_common::{
    model::puzzle::{JigsawTile, PublicJigsawTile},
    redis_scheme::RedisScheme,
};
use redis::AsyncCommands;
use tokio::sync::broadcast::Receiver;
use uuid::Uuid;

use crate::model::user::User;

use super::{message::WsMessage, state::AppState, unauthorized_handler::UnauthorizedSocketHandler};

pub struct SocketHandler {
    user: User,
    socket: WebSocket,
    puzzle_uuid: Uuid,
    state: AppState,
}

use eyre::Report;

impl SocketHandler {
    pub fn new(handler: UnauthorizedSocketHandler, user: User) -> Self {
        Self {
            user,
            socket: handler.socket,
            puzzle_uuid: handler.puzzle_uuid,
            state: handler.state,
        }
    }

    async fn get_initial_data(&mut self) -> Result<HashMap<Uuid, PublicJigsawTile>, Report> {
        let key = RedisScheme::jigsaw_puzzle_state(&self.puzzle_uuid);

        let redis = self.state.redis.clone();

        let raw_data: HashMap<String, Vec<u8>> = self.state.redis.hgetall(key).await?;
        if raw_data.is_empty() {
            Err(eyre::eyre!("No puzzle with such UUID"))?;
        }

        let data = raw_data
            .into_iter()
            .map(|(key, value)| {
                Ok((
                    Uuid::from_str(&key)?,
                    rmp_serde::from_slice::<JigsawTile>(&value)?.into(),
                ))
            })
            .collect::<Result<HashMap<Uuid, PublicJigsawTile>, Report>>()?;

        Ok(data)
    }

    async fn process_message_task(
        mut ch_receiver: Receiver<Message>,
        mut ws_sender: SplitSink<WebSocket, Message>,
    ) -> Result<(), Report> {
        while let Ok(message) = ch_receiver.recv().await {
            let _ = ws_sender.send(message).await;
        }
        Ok(())
    }

    pub async fn handle(mut self) {
        let initial_data = self.get_initial_data().await.unwrap();

        let initial_message = WsMessage::Initial { data: initial_data };

        self.socket
            .send(initial_message.try_into().unwrap())
            .await
            .unwrap();

        let (ws_sender, _) = self.socket.split();

        let (ch_receiver, _) = self.state.get_channel(&self.puzzle_uuid).await;

        let process_message_task = tokio::spawn(async move {
            Self::process_message_task(ch_receiver, ws_sender)
                .await
                .unwrap()
        });

        let _ = process_message_task.await;

        self.state.remove_if_no_receivers(&self.puzzle_uuid).await;
    }
}
