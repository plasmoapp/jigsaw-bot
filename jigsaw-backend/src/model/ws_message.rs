use std::collections::HashMap;

use axum::extract::ws::Message;
use jigsaw_common::model::puzzle::{JigsawIndex, PublicJigsawTile};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type")]
pub enum WsMessage {
    Initial {
        data: HashMap<Uuid, PublicJigsawTile>,
    },
    Placed {
        // user: UserId,
        tile_uuid: Uuid,
        index: JigsawIndex,
    },
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type")]
pub enum WsRequest {
    Place { tile_uuid: Uuid, index: JigsawIndex },
}

impl TryFrom<&WsMessage> for Message {
    type Error = serde_json::Error;

    fn try_from(value: &WsMessage) -> Result<Self, Self::Error> {
        let ser = serde_json::to_string(value)?;
        Ok(Self::Text(ser))
    }
}

impl TryFrom<WsMessage> for Message {
    type Error = serde_json::Error;

    fn try_from(value: WsMessage) -> Result<Self, Self::Error> {
        (&value).try_into()
    }
}
