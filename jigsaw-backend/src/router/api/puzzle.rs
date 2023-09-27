use std::{collections::HashMap, str::FromStr, sync::Arc};

use axum::{extract::Path, Extension, Json};
use eyre::Report;
use jigsaw_common::{
    model::puzzle::{JigsawIndex, JigsawTile, PublicJigsawTile},
    redis_scheme::RedisScheme,
};
use redis::{aio::MultiplexedConnection, AsyncCommands};

use crate::{error::ReportResposnse, model::ws_message::WsMessage, ws_state::WsState};

use uuid::Uuid;

// pub async fn post_place(
//     Path((puzzle_uuid, tile_uuid)): Path<(Uuid, Uuid)>,
//     Extension(mut redis_connection): Extension<MultiplexedConnection>,
//     Extension(ws_state): Extension<Arc<WsState>>,
//     Json(index): Json<JigsawIndex>,
// ) -> Result<Json<bool>, ReportResposnse> {
//     let key = RedisScheme::jigsaw_puzzle_state(&puzzle_uuid);
//     let field = tile_uuid.to_string();
//     let raw_tile_data = redis_connection
//         .hget::<'_, _, _, Option<Vec<u8>>>(&key, &field)
//         .await?
//         .ok_or(eyre::eyre!("No tile with such Uuid"))?;
//     let mut tile_data: JigsawTile = rmp_serde::from_slice::<JigsawTile>(&raw_tile_data)?;

//     // dbg!(&tile_data);

//     if tile_data.in_place {
//         return Ok(Json(true));
//     }

//     if tile_data.index != index {
//         return Ok(Json(false));
//     }

//     tile_data.in_place = true;

//     // dbg!(&tile_data);

//     redis_connection
//         .hset(&key, &field, rmp_serde::to_vec(&tile_data)?)
//         .await?;

//     if let Some(sender) = ws_state.get_sender(&puzzle_uuid).await {
//         let message = WsMessage::Placed { tile_uuid, index };
//         let _ = sender.send(Arc::new(message));
//     };
//     Ok(Json(true))
// }
