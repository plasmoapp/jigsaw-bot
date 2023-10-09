use std::collections::HashMap;

use axum::extract::ws::WebSocket;
use chrono::{Duration, NaiveDateTime, Utc};
use futures::StreamExt;
use itertools::Itertools;

use pipe_trait::Pipe;
use redis::AsyncCommands;

use uuid::Uuid;

use crate::model::user::{TelegramUser, User};

use super::{error::SocketError, handler::SocketHandler, message::WsRequest, state::AppState};

pub struct UnauthorizedSocketHandler {
    pub socket: WebSocket,
    pub puzzle_uuid: Uuid,
    pub state: AppState,
}

// In the debug mode our server supports authorization protocol that doesn't require user to autorize with Telegram
// This is only used for testing purposes. In release mode this protocol is not available
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

        let result = SocketHandler::new(self, user);

        Ok(result)
    }

    // Based on https://core.telegram.org/bots/webapps#validating-data-received-via-the-mini-app
    async fn authorize_telegram(&mut self) -> Result<User, SocketError> {
        // With this protocol we extect client to send TelegramAuth request with initData as the first packet
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

        // initData is a urlencoded query stirng so we need to decode it
        let init_data = urlencoding::decode(&init_data)?;

        // Transform initData into a HashMap we can use to get the values we need
        let query = init_data
            .split('&')
            .flat_map(|str| str.split_once('='))
            .collect::<HashMap<_, _>>();

        // data_check_string is used to verify the initData using hash
        let data_check_string = init_data
            .split('&')
            // This is not explicitly stated in the documentation, but we actually need to filter out the 'hash' field
            // You can figure it out because it's impossible to include hash in the hashed message
            .filter(|str| !str.starts_with("hash"))
            // Sort alphabetically like stated in the docs
            .sorted_by(|a, b| Ord::cmp(a, b))
            .intersperse("\n")
            .collect::<String>();

        // telegram_web_secret is an equivalent of 'HMAC_SHA256(<bot_token>, "WebAppData")' from the documentation
        // This variable is equivalent of HMAC_SHA256(data_check_string, secret_key)
        let tag = ring::hmac::sign(
            &self.state.telegram_web_secret,
            data_check_string.as_bytes(),
        );

        let hash = query.get("hash").ok_or(SocketError::InvalidCredentials)?;

        // Encode the hash that we got and compare it to the hash from initData
        let verify = &hex::encode(tag.as_ref()) == hash;

        if !verify {
            Err(SocketError::InvalidCredentials)?;
        }

        // Parse auth_data field from initData
        let auth_date = query
            .get("auth_date")
            .and_then(|date| date.parse::<i64>().ok())
            .and_then(|date| NaiveDateTime::from_timestamp_opt(date, 0))
            .expect("auth_date can't be invalid after the data was verified");

        let now = Utc::now().naive_utc();

        // Return Err if initData is older than 15 minutes
        if now.signed_duration_since(auth_date) > Duration::minutes(15) {
            Err(SocketError::CredentialsExpired)?;
        }

        let user: TelegramUser = query
            .get("user")
            .expect("User can't be None after the data was verified")
            .pipe(|user| serde_json::from_str(user))?;

        // Now we can finaly return the User after making sure that all the data is valid
        Ok(user.into())
    }
}
