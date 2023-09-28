use std::{collections::HashMap, str::FromStr, sync::Arc};

use axum::extract::ws::{Message, WebSocket};
use futures::{stream::SplitSink, SinkExt, StreamExt};
use itertools::Itertools;
use jigsaw_common::{
    model::puzzle::{JigsawTile, PublicJigsawTile},
    redis_scheme::RedisScheme,
};
use redis::{aio::MultiplexedConnection, AsyncCommands};
use tokio::sync::broadcast::Receiver;
use uuid::Uuid;

use crate::{config::Config, model::user::User};

use eyre::Report;

use super::{
    error::SocketAuthError,
    message::{WsMessage, WsRequest},
    state::WebSocketState,
};

use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

pub struct SocketHandler {
    socket: WebSocket,
    puzzle_uuid: Uuid,
    state: Arc<WebSocketState>,
    config: Arc<Config>,
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
        config: Arc<Config>,
        redis: MultiplexedConnection,
    ) -> Self {
        Self {
            socket,
            puzzle_uuid,
            config,
            state,
            redis,
        }
    }

    async fn authorize(socket: &mut WebSocket, config: &Config) -> Result<User, SocketAuthError> {
        let protocol = socket
            .protocol()
            .ok_or(SocketAuthError::NoProtocol)?
            .to_str()?;

        let user = match protocol {
            Self::TELEGRAM_AUTH_PROTOCOL => Self::authorize_telegram(socket, config).await?,
            #[cfg(debug_assertions)]
            Self::NOT_SECURE_PROTOCOL => User::test(),
            protocol => Err(SocketAuthError::UnsupportedProtocol(protocol.into()))?,
        };

        Ok(user)
    }

    async fn authorize_telegram(
        socket: &mut WebSocket,
        config: &Config,
    ) -> Result<User, SocketAuthError> {
        // let Some(message) = socket.recv().await else {
        //     return Err(SocketAuthError::SocketClosed);
        // };

        let message: WsRequest = socket
            .recv()
            .await
            .ok_or(SocketAuthError::SocketClosed)??
            .try_into()?;

        let init_data = match message {
            WsRequest::TelegramAuth { data_check_string } => data_check_string,
            _ => return Err(SocketAuthError::InvalidRequest("TelegramAuth".into())),
        };

        let init_data = urlencoding::decode(&init_data).unwrap();

        dbg!(&init_data);

        let mut query = init_data
            .split('&')
            .flat_map(|str| str.split_once('='))
            .collect::<HashMap<_, _>>();

        let hash = query
            .remove("hash")
            .ok_or(SocketAuthError::InvalidCredentials)?;

        let data_check_string = query
            .iter()
            .sorted_by(|(a, _), (b, _)| Ord::cmp(a, b))
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join("\n");

        // let data_check_string = init_data
        //     .split('&')
        //     .filter(|str| !str.starts_with("hash"))
        //     .sorted_by(|a, b| Ord::cmp(a, b))
        //     .intersperse("\n")
        //     .collect::<String>();

        dbg!(&config.bot_token);

        dbg!(&data_check_string);

        let tag = ring::hmac::sign(&key, data_check_string.as_bytes());

        dbg!(hex::encode(tag.as_ref()));
        dbg!(hash);

        // secret_key.as_ref()

        // let map = data_check_string
        //     .split('\n')
        //     .flat_map(|str| str.split_once('='))
        //     .collect::<HashMap<_, _>>();

        // dbg!(&map);

        // let mut mac = HmacSha256::new_varkey(config.bot_token.as_bytes())
        //     .expect("HMAC can take key of any size");
        // mac.update(b"WebAppData");

        // let secret_key = mac.finalize().into_bytes();

        // let mut mac = HmacSha256::new_from_slice(data_check_string.as_bytes())
        //     .expect("HMAC can take key of any size");
        // mac.update(&secret_key);

        // let verify_hash = mac.finalize().into_bytes();

        // dbg!(hex::encode(verify_hash));

        // dbg!(hash);

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
        mut ch_receiver: Receiver<Message>,
        mut ws_sender: SplitSink<WebSocket, Message>,
    ) -> Result<(), Report> {
        while let Ok(message) = ch_receiver.recv().await {
            let _ = ws_sender.send(message).await;
        }
        Ok(())
    }

    pub async fn handle(mut self) {
        // match protocol {
        //     Self::
        //     _ => return,
        // }

        // let user = self.authorize().await.unwrap();

        let user = Self::authorize(&mut self.socket, &self.config)
            .await
            .unwrap();

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
