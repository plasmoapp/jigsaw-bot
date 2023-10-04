use std::{collections::HashMap};

use axum::extract::ws::{WebSocket};
use chrono::{Duration, NaiveDateTime, Utc};
use futures::{StreamExt};
use itertools::Itertools;

use pipe_trait::Pipe;
use redis::{AsyncCommands};

use uuid::Uuid;

use crate::{
    model::user::{TelegramUser, User},
};



use super::{
    error::SocketError,
    handler::SocketHandler,
    message::{WsRequest},
    state::AppState,
};

pub struct UnauthorizedSocketHandler {
    pub socket: WebSocket,
    pub puzzle_uuid: Uuid,
    pub state: AppState,
}

const TELEGRAM_AUTH_PROTOCOL: &str = "jigsaw-telegram-auth";
#[cfg(debug_assertions)]
const NOT_SECURE_PROTOCOL: &str = "jigsaw-not-secure";
#[cfg(debug_assertions)]
pub const PROTOCOLS: [&str; 2] = [TELEGRAM_AUTH_PROTOCOL, NOT_SECURE_PROTOCOL];
#[cfg(not(debug_assertions))]
pub const PROTOCOLS: [&str; 1] = [TELEGRAM_AUTH_PROTOCOL];

impl UnauthorizedSocketHandler {
    pub fn new(socket: WebSocket, puzzle_uuid: Uuid, state: AppState) -> Self {
        Self {
            socket,
            puzzle_uuid,
            state,
        }
    }

    pub async fn authorize(mut self) -> Result<SocketHandler, SocketError> {
        let protocol = self
            .socket
            .protocol()
            .ok_or(SocketError::NoProtocol)?
            .to_str()?;

        let user = match protocol {
            TELEGRAM_AUTH_PROTOCOL => self.authorize_telegram().await?,
            #[cfg(debug_assertions)]
            NOT_SECURE_PROTOCOL => User::test(),
            protocol => Err(SocketError::UnsupportedProtocol(protocol.into()))?,
        };

        // println!("Authorized as {}", user.1.name);

        let result = SocketHandler::new(self, user);

        Ok(result)
    }

    async fn authorize_telegram(&mut self) -> Result<User, SocketError> {
        let message: WsRequest = self
            .socket
            .recv()
            .await
            .ok_or(SocketError::SocketClosed)??
            .try_into()?;

        let init_data = match message {
            WsRequest::TelegramAuth { init_data } => init_data,
            _ => return Err(SocketError::InvalidRequest("TelegramAuth".into())),
        };

        let init_data = urlencoding::decode(&init_data)?;

        let query = init_data
            .split('&')
            .flat_map(|str| str.split_once('='))
            .collect::<HashMap<_, _>>();

        let data_check_string = init_data
            .split('&')
            .filter(|str| !str.starts_with("hash"))
            .sorted_by(|a, b| Ord::cmp(a, b))
            .intersperse("\n")
            .collect::<String>();

        let tag = ring::hmac::sign(
            &self.state.telegram_web_secret,
            data_check_string.as_bytes(),
        );

        let hash = query.get("hash").ok_or(SocketError::InvalidCredentials)?;

        let verify = &hex::encode(tag.as_ref()) == hash;

        if !verify {
            Err(SocketError::InvalidCredentials)?;
        }

        let auth_date = query
            .get("auth_date")
            .and_then(|date| date.parse::<i64>().ok())
            .and_then(|date| NaiveDateTime::from_timestamp_opt(date, 0))
            .ok_or(SocketError::InvalidCredentials)?;

        let now = Utc::now().naive_utc();

        if now.signed_duration_since(auth_date) > Duration::minutes(15) {
            Err(SocketError::CredentialsExpired)?;
        }

        let user: TelegramUser = query
            .get("user")
            .expect("User can't be None after the data was verified")
            .pipe(|user| serde_json::from_str(user))?;

        Ok(user.into())
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
