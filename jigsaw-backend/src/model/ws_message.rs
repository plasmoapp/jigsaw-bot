use std::collections::HashMap;

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
        tile_uuid: Uuid,
        index: JigsawIndex,
    },
}
