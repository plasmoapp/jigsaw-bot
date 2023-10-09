use std::{collections::HashMap, str::FromStr};

use axum::extract::ws::{Message, WebSocket};
use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use jigsaw_common::{
    model::puzzle::{JigsawIndex, JigsawMeta, JigsawTile, PublicJigsawTile},
    redis_scheme::RedisScheme,
};
use redis::{aio::MultiplexedConnection, AsyncCommands};
use tokio::sync::broadcast::{Receiver, Sender};
use uuid::Uuid;

use crate::model::user::{User, UserData, UserId};

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
        let score_key = RedisScheme::jigsaw_puzzle_score(puzzle_uuid);

        let (raw_state, raw_meta, raw_score): (HashMap<String, Vec<u8>>, Vec<u8>, Vec<(u64, u64)>) =
            redis::pipe()
                .hgetall(state_key)
                .get(meta_key)
                .zrange_withscores(&score_key, 0, -1)
                .query_async(redis)
                .await?;

        let user_ids = raw_score
            .iter()
            .map(|(key, _)| key)
            .copied()
            .collect::<Vec<_>>();

        let raw_users: Vec<Vec<u8>> = redis.hget(RedisScheme::JIGSAW_USER_DATA, &user_ids).await?;

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

        let users = user_ids
            .into_iter()
            .zip(raw_users)
            .map(|(key, value)| Ok((UserId(key), rmp_serde::from_slice(&value)?)))
            .collect::<Result<HashMap<UserId, UserData>, Report>>()?;

        let scores = raw_score
            .into_iter()
            .map(|(key, value)| (UserId(key), value))
            .collect();

        let message = WsMessage::Initial {
            state,
            meta,
            users,
            scores,
        };

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

    async fn process_place(
        puzzle_uuid: &Uuid,
        user: &User,
        tile_uuid: &Uuid,
        index: JigsawIndex,
        redis: &mut MultiplexedConnection,
        ch_sender: &mut Sender<Message>,
    ) -> Result<(), Report> {
        let key = RedisScheme::jigsaw_puzzle_state(puzzle_uuid);
        let field = tile_uuid.to_string();
        let raw_tile_data = redis
            .hget::<'_, _, _, Option<Vec<u8>>>(&key, &field)
            .await?
            .ok_or(eyre::eyre!("No tile with such Uuid"))?;
        let mut tile_data: JigsawTile = rmp_serde::from_slice::<JigsawTile>(&raw_tile_data)?;

        if tile_data.in_place || tile_data.index != index {
            return Ok::<_, Report>(());
        }

        tile_data.in_place = true;

        redis::pipe()
            .hset(&key, &field, rmp_serde::to_vec(&tile_data)?)
            .zincr(RedisScheme::jigsaw_puzzle_score(puzzle_uuid), *user.0, 1)
            .incr(RedisScheme::jigsaw_puzzle_score_total(puzzle_uuid), 1)
            .query_async(redis)
            .await?;

        WsMessage::placed(user.0, tile_uuid.to_owned(), index).send_ch(ch_sender)?;

        Ok(())
    }

    async fn set_user_data(
        user: &User,
        puzzle_uuid: &Uuid,
        redis: &mut MultiplexedConnection,
    ) -> Result<(), Report> {
        redis::pipe()
            .hset(
                RedisScheme::JIGSAW_USER_DATA,
                *user.0,
                rmp_serde::to_vec(&user.1)?,
            )
            .zincr(RedisScheme::jigsaw_puzzle_score(puzzle_uuid), *user.0, 0)
            .query_async(redis)
            .await?;
        Ok(())
    }

    async fn process_request_task(
        mut ch_sender: Sender<Message>,
        mut ws_receiver: SplitStream<WebSocket>,
        user: User,
        puzzle_uuid: Uuid,
        state: AppState,
    ) {
        let mut redis = state.redis.clone();

        while let Some(Ok(message)) = ws_receiver.next().await {
            let request = match WsRequest::try_from(message) {
                Ok(v) => v,
                Err(error) => {
                    tracing::warn!("Invalid WebSocket request: {error}");
                    continue;
                }
            };

            let result = match request {
                WsRequest::Place { tile_uuid, index } => {
                    Self::process_place(
                        &puzzle_uuid,
                        &user,
                        &tile_uuid,
                        index,
                        &mut redis,
                        &mut ch_sender,
                    )
                    .await
                }
                WsRequest::Chat { message: _ } => Ok(()),
                _ => Ok(()),
            };

            if let Err(error) = result {
                tracing::error!("Error while processing request: {error}");
            };
        }
    }

    pub async fn handle(mut self) {
        let (mut ws_sender, ws_receiver) = self.socket.split();

        let (ch_receiver, ch_sender) = self.state.get_channel(&self.puzzle_uuid).await;

        if let Err(error) =
            Self::set_user_data(&self.user, &self.puzzle_uuid, &mut self.state.redis).await
        {
            tracing::error!("Error while setting user data: {error}");
            return;
        }

        let initial_message =
            match Self::get_initial_data(&self.puzzle_uuid, &mut self.state.redis).await {
                Ok(m) => m,
                Err(error) => {
                    tracing::error!("Error while getting the initial data: {error}");
                    return;
                }
            };

        if let Err(error) = ws_sender.send(initial_message).await {
            tracing::error!("Error while sending the initial data: {error}");
            return;
        }

        let _ = WsMessage::join(self.user.clone()).send_ch(&ch_sender);

        let process_message_task =
            tokio::spawn(async move { Self::process_message_task(ch_receiver, ws_sender).await });

        let ch_sender_clone = ch_sender.clone();
        let user_clone = self.user.clone();
        let puzzle_uuid_clone = self.puzzle_uuid;
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

        let _ = WsMessage::leave(self.user.0).send_ch(&ch_sender);

        self.state.remove_if_no_receivers(&self.puzzle_uuid).await;
    }
}
