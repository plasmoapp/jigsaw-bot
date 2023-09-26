use axum::{
    extract::{
        ws::{Message, WebSocket},
        Path, WebSocketUpgrade,
    },
    response::IntoResponse,
    Extension,
};
use eyre::Report;
use jigsaw_common::{
    model::puzzle::{JigsawTile, PublicJigsawTile},
    redis_scheme::RedisScheme,
};
use redis::{aio::MultiplexedConnection, AsyncCommands};
use std::{collections::HashMap, str::FromStr, sync::Arc};
use tokio::sync::broadcast::{Receiver, Sender};
use uuid::Uuid;

use futures::{
    sink::SinkExt,
    stream::{SplitSink, SplitStream, StreamExt},
};

use crate::{model::ws_message::WsMessage, ws_state::WsState};

pub async fn get_ws(
    ws: WebSocketUpgrade,
    Path(puzzle_uuid): Path<Uuid>,
    Extension(ws_state): Extension<Arc<WsState>>,
    Extension(redis_connection): Extension<MultiplexedConnection>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, puzzle_uuid, ws_state, redis_connection))
}

async fn handle_socket(
    mut socket: WebSocket,
    puzzle_uuid: Uuid,
    ws_state: Arc<WsState>,
    mut redis_connection: MultiplexedConnection,
) {
    let initial_data = get_initial_data(&puzzle_uuid, &mut redis_connection)
        .await
        .unwrap();

    let initial_message = WsMessage::Initial { data: initial_data };

    socket
        .send(Message::Text(
            serde_json::to_string(&initial_message).unwrap(),
        ))
        .await
        .unwrap();

    let (ws_sender, _) = socket.split();

    let (ch_receiver, _) = ws_state.get_channel(&puzzle_uuid).await;

    // let mut process_request_task = tokio::spawn(async move {
    //     process_request_task(&puzzle_uuid, ws_receiver, ch_sender, redis_connection)
    //         .await
    //         .unwrap()
    // });

    let process_message_task =
        tokio::spawn(async move { process_message_task(ch_receiver, ws_sender).await.unwrap() });

    let _ = process_message_task.await;

    ws_state.remove_if_no_receivers(&puzzle_uuid);

    // tokio::select! {x
    //     _ = (&mut process_request_task) => {
    //         process_message_task.abort();
    //     },
    //     _ = (&mut process_message_task) => {
    //         process_request_task.abort();
    //     }
    // }
}

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

async fn process_message_task(
    mut ch_receiver: Receiver<Arc<WsMessage>>,
    mut ws_sender: SplitSink<WebSocket, Message>,
) -> Result<(), Report> {
    while let Ok(message) = ch_receiver.recv().await {
        let _ = ws_sender
            .send(Message::Text(
                serde_json::to_string(message.as_ref()).unwrap(),
            ))
            .await;
    }
    Ok(())
}

async fn get_initial_data(
    puzzle_uuid: &Uuid,
    redis_connection: &mut MultiplexedConnection,
) -> Result<HashMap<Uuid, PublicJigsawTile>, Report> {
    let key = RedisScheme::jigsaw_puzzle_state(puzzle_uuid);

    let raw_data: HashMap<String, Vec<u8>> = redis_connection.hgetall(key).await?;
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
