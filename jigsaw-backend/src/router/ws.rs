use std::sync::Arc;

use axum::{
    extract::{Path, WebSocketUpgrade},
    response::IntoResponse,
    Extension,
};
use redis::aio::MultiplexedConnection;
use uuid::Uuid;

use crate::websocket::{handler::SocketHandler, state::WebSocketState};

pub async fn get_ws(
    websocket: WebSocketUpgrade,
    Path(puzzle_uuid): Path<Uuid>,
    Extension(ws_state): Extension<Arc<WebSocketState>>,
    Extension(redis_connection): Extension<MultiplexedConnection>,
) -> impl IntoResponse {
    websocket
        .protocols(SocketHandler::PROTOCOLS)
        .on_upgrade(move |socket| {
            SocketHandler::new(socket, puzzle_uuid, ws_state, redis_connection).handle()
        })
}
