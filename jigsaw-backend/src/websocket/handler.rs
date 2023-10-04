use std::{collections::HashMap, str::FromStr};

use axum::extract::ws::{Message, WebSocket};
use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use jigsaw_common::{
    model::puzzle::{JigsawMeta, JigsawTile, PublicJigsawTile},
    redis_scheme::RedisScheme,
};
use redis::{aio::MultiplexedConnection, AsyncCommands};
use tokio::sync::broadcast::{Receiver, Sender};
use uuid::Uuid;

use crate::model::user::{User, UserId};

use super::{
    message::{WsMessage, WsRequest},
    state::AppState,
    unauthorized_handler::UnauthorizedSocketHandler,
};

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

    async fn get_initial_data(
        puzzle_uuid: &Uuid,
        redis: &mut MultiplexedConnection,
    ) -> Result<Message, Report> {
        let state_key = RedisScheme::jigsaw_puzzle_state(puzzle_uuid);
        let meta_key = RedisScheme::jigsaw_puzzle_meta(puzzle_uuid);

        let (raw_state, raw_meta): (HashMap<String, Vec<u8>>, Vec<u8>) = redis::pipe()
            .hgetall(state_key)
            .get(meta_key)
            .query_async(redis)
            .await?;

        if raw_state.is_empty() {
            Err(eyre::eyre!("No puzzle with such UUID"))?;
        }

        let state = raw_state
            .into_iter()
            .map(|(key, value)| {
                Ok((
                    Uuid::from_str(&key)?,
                    rmp_serde::from_slice::<JigsawTile>(&value)?.into(),
                ))
            })
            .collect::<Result<HashMap<Uuid, PublicJigsawTile>, Report>>()?;

        let meta = rmp_serde::from_slice::<JigsawMeta>(&raw_meta)?;

        let message = WsMessage::Initial { state, meta };

        Ok(message.try_into()?)
    }

    async fn process_message_task(
        mut ch_receiver: Receiver<Message>,
        mut ws_sender: SplitSink<WebSocket, Message>,
    ) {
        while let Ok(message) = ch_receiver.recv().await {
            let _ = ws_sender.send(message).await;
        }
    }

    async fn process_request_task(
        mut ch_sender: Sender<Message>,
        mut ws_receiver: SplitStream<WebSocket>,
        user: User,
        _puzzle_uuid: Uuid,
        _state: AppState,
    ) {
        while let Some(Ok(message)) = ws_receiver.next().await {
            let request = match WsRequest::try_from(message) {
                Ok(v) => v,
                Err(error) => {
                    tracing::warn!("Invalid WebSocket request: {error}");
                    continue;
                }
            };

            match request {
                WsRequest::Place { tile_uuid, index } => {}
                WsRequest::Chat { message } => {
                    if let Ok(chat_message) = (WsMessage::Chat {
                        user: user.0,
                        message,
                    })
                    .try_into()
                    {
                        let _ = ch_sender.send(chat_message);
                    };
                }
                _ => {}
            }
        }
    }

    pub async fn handle(mut self) {
        let (mut ws_sender, ws_receiver) = self.socket.split();

        let (ch_receiver, ch_sender) = self.state.get_channel(&self.puzzle_uuid).await;

        let initial_message =
            match Self::get_initial_data(&self.puzzle_uuid, &mut self.state.redis).await {
                Ok(m) => m,
                Err(error) => {
                    tracing::error!("Error while getting the initial data: {error}");
                    return;
                }
            };

        let _ = ws_sender.send(initial_message).await;

        if let Ok(join_message) = (WsMessage::Join {
            user: self.user.clone(),
        })
        .try_into()
        {
            let _ = ch_sender.send(join_message);
        }

        let process_message_task =
            tokio::spawn(async move { Self::process_message_task(ch_receiver, ws_sender).await });

        let ch_sender_clone = ch_sender.clone();
        let user_clone = self.user.clone();
        let puzzle_uuid_clone = self.puzzle_uuid.clone();
        let state_clone = self.state.clone();

        let process_request_task = tokio::spawn(async move {
            Self::process_request_task(
                ch_sender_clone,
                ws_receiver,
                user_clone,
                puzzle_uuid_clone,
                state_clone,
            )
            .await
        });

        tokio::select! {
            _ = process_message_task => {},
            _ = process_request_task => {},
        }

        if let Ok(leave_message) = (WsMessage::Leave { user: self.user.0 }).try_into() {
            let _ = ch_sender.send(leave_message);
        }

        self.state.remove_if_no_receivers(&self.puzzle_uuid).await;
    }
}
