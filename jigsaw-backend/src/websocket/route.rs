use std::sync::Arc;

use axum::{
    extract::{Path, State, WebSocketUpgrade},
    response::IntoResponse,
    Extension,
};
use redis::aio::MultiplexedConnection;
use uuid::Uuid;

use crate::{
    config::Config,
    websocket::{handler::SocketHandler, state::AppState},
};

use super::unauthorized_handler::{UnauthorizedSocketHandler, PROTOCOLS};

pub async fn get_puzzle_websocket(
    websocket: WebSocketUpgrade,
    Path(puzzle_uuid): Path<Uuid>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    websocket
        .protocols(PROTOCOLS)
        .on_upgrade(move |socket| async move {
            match UnauthorizedSocketHandler::new(socket, puzzle_uuid, state)
                .authorize()
                .await
            {
                Ok(handler) => handler.handle().await,
                Err(error) => tracing::error!("Failed to authorize socket: {error}"),
            };
        })
}
