use std::{collections::HashMap, str::FromStr, sync::Arc};

use axum::extract::ws::{Message, WebSocket};
use futures::{stream::SplitSink, SinkExt, StreamExt};
use jigsaw_common::{
    model::puzzle::{JigsawTile, PublicJigsawTile},
    redis_scheme::RedisScheme,
};
use redis::{aio::MultiplexedConnection, AsyncCommands};
use tokio::sync::broadcast::Receiver;
use uuid::Uuid;

use crate::model::ws_message::WsMessage;

use eyre::Report;

use super::{error::SocketAuthError, state::WebSocketState};

pub struct SocketHandler {
    socket: WebSocket,
    puzzle_uuid: Uuid,
    state: Arc<WebSocketState>,
    redis: MultiplexedConnection,
}

impl SocketHandler {
    const TELEGRAM_AUTH_PROTOCOL: &str = "jigsaw-telegram-auth";

    #[cfg(debug_assertions)]
    const NOT_SECURE_PROTOCOL: &str = "jigsaw-not-secure";

    #[cfg(debug_assertions)]
    pub const PROTOCOLS: [&str; 2] = [Self::TELEGRAM_AUTH_PROTOCOL, Self::NOT_SECURE_PROTOCOL];

    #[cfg(not(debug_assertions))]
    pub const PROTOCOLS: [&str; 1] = [Self::TELEGRAM_AUTH_PROTOCOL];

    pub fn new(
        socket: WebSocket,
        puzzle_uuid: Uuid,
        state: Arc<WebSocketState>,
        redis: MultiplexedConnection,
    ) -> Self {
        Self {
            socket,
            puzzle_uuid,
            state,
            redis,
        }
    }

    async fn authorize(&mut self) -> Result<(), SocketAuthError> {
        let protocol = self
            .socket
            .protocol()
            .ok_or(SocketAuthError::NoProtocol)?
            .to_str()?;

        let user = match protocol {
            Self::TELEGRAM_AUTH_PROTOCOL => "Pepega2",
            #[cfg(debug_assertions)]
            Self::NOT_SECURE_PROTOCOL => "Pepega",
            protocol => Err(SocketAuthError::UnsupportedProtocol(protocol.into()))?,
        };

        todo!()
    }

    async fn authorize_telegram(&mut self) -> Result<(), SocketAuthError> {
        todo!()
    }

    async fn get_initial_data(&mut self) -> Result<HashMap<Uuid, PublicJigsawTile>, Report> {
        let key = RedisScheme::jigsaw_puzzle_state(&self.puzzle_uuid);

        let raw_data: HashMap<String, Vec<u8>> = self.redis.hgetall(key).await?;
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
        mut ch_receiver: Receiver<Arc<WsMessage>>,
        mut ws_sender: SplitSink<WebSocket, Message>,
    ) -> Result<(), Report> {
        while let Ok(message) = ch_receiver.recv().await {
            let _ = ws_sender.send(message.as_ref().try_into().unwrap()).await;
        }
        Ok(())
    }

    pub async fn handle(mut self) {
        // match protocol {
        //     Self::
        //     _ => return,
        // }

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

// async fn handle_socket(
//     mut socket: WebSocket,
//     puzzle_uuid: Uuid,
//     ws_state: Arc<WsState>,
//     mut redis_connection: MultiplexedConnection,
// ) {
// let mut process_request_task = tokio::spawn(async move {
//     process_request_task(&puzzle_uuid, ws_receiver, ch_sender, redis_connection)
//         .await
//         .unwrap()
// });

// tokio::select! {x
//     _ = (&mut process_request_task) => {
//         process_message_task.abort();
//     },
//     _ = (&mut process_message_task) => {
//         process_request_task.abort();
//     }
// }
// }

// async fn process_request_task(
//     puzzle_uuid: &Uuid,
//     mut ws_receiver: SplitStream<WebSocket>,
//     ch_sender: Sender<Arc<WsMessage>>,
//     mut redis_connection: MultiplexedConnection,
// ) -> Result<(), Report> {
//     while let Some(Ok(message)) = ws_receiver.next().await {
//         process_request(puzzle_uuid, message, &mut redis_connection)
//             .await
//             .unwrap();
//     }
//     Ok(())
// }
