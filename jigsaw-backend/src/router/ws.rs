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

pub async fn get_puzzle_websocket(
    websocket: WebSocketUpgrade,
    Path(puzzle_uuid): Path<Uuid>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    websocket
        .protocols(SocketHandler::PROTOCOLS)
        .on_upgrade(move |socket| async {
            // SocketHandler::new(socket, puzzle_uuid, ws_state, config, redis_connection).handle()
        })
}
