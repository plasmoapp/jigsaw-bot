use std::collections::HashMap;

use axum::extract::ws::Message;
use jigsaw_common::model::puzzle::{JigsawIndex, JigsawMeta, PublicJigsawTile};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::model::user::{User, UserId};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type")]
pub enum WsMessage {
    Initial {
        meta: JigsawMeta,
        state: HashMap<Uuid, PublicJigsawTile>,
    },
    Placed {
        user: UserId,
        tile_uuid: Uuid,
        index: JigsawIndex,
    },
    Join {
        user: User,
    },
    Leave {
        user: UserId,
    },
    Chat {
        user: UserId,
        message: String,
    },
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum WsRequest {
    TelegramAuth { init_data: String },
    Place { tile_uuid: Uuid, index: JigsawIndex },
    Chat { message: String },
}

impl TryFrom<Message> for WsRequest {
    type Error = serde_json::Error;

    fn try_from(value: Message) -> Result<Self, Self::Error> {
        let de = serde_json::from_slice(&value.into_data())?;
        Ok(de)
    }
}

impl TryFrom<WsMessage> for Message {
    type Error = serde_json::Error;

    fn try_from(value: WsMessage) -> Result<Self, Self::Error> {
        let ser = serde_json::to_string(&value)?;
        Ok(Self::Text(ser))
    }
}
