

use axum::{
    extract::{Path, State, WebSocketUpgrade},
    response::IntoResponse,
};

use uuid::Uuid;

use crate::{
    websocket::{state::AppState},
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
