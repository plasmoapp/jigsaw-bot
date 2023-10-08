use std::collections::HashMap;

use axum::extract::ws::Message;
use jigsaw_common::model::puzzle::{JigsawIndex, JigsawMeta, PublicJigsawTile};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast::Sender;
use uuid::Uuid;

use crate::model::user::{User, UserData, UserId};

use eyre::Report;

// Message is Server to Client
// Request is Client to Server

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type")]
pub enum WsMessage {
    Initial {
        meta: JigsawMeta,
        state: HashMap<Uuid, PublicJigsawTile>,
        users: HashMap<UserId, UserData>,
        scores: HashMap<UserId, u64>,
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

impl WsMessage {
    pub fn initial(
        meta: JigsawMeta,
        state: HashMap<Uuid, PublicJigsawTile>,
        users: HashMap<UserId, UserData>,
        scores: HashMap<UserId, u64>,
    ) -> Self {
        Self::Initial {
            meta,
            state,
            users,
            scores,
        }
    }

    pub fn placed(user: UserId, tile_uuid: Uuid, index: JigsawIndex) -> Self {
        Self::Placed {
            user,
            tile_uuid,
            index,
        }
    }

    pub fn join(user: User) -> Self {
        Self::Join { user }
    }

    pub fn leave(user: UserId) -> Self {
        Self::Leave { user }
    }

    pub fn send_ch(self, sender: &Sender<Message>) -> Result<(), Report> {
        let message: Message = self.try_into()?;
        sender.send(message)?;
        Ok(())
    }
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
