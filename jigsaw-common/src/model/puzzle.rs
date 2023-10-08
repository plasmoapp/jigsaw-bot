use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use shrinkwraprs::Shrinkwrap;
use uuid::Uuid;

#[derive(Debug)]
pub struct JigsawPuzzle {
    pub uuid: Uuid,
    pub tile_map: HashMap<Uuid, JigsawTile>,
    pub meta: JigsawMeta,
}

impl JigsawPuzzle {
    pub fn new(uuid: Uuid, tile_map: HashMap<Uuid, JigsawTile>, meta: JigsawMeta) -> Self {
        Self {
            uuid,
            tile_map,
            meta,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JigsawMeta {
    pub tile_size_px: u32,
    pub image_dimensions_px: (u32, u32),
}

impl JigsawMeta {
    pub fn new(tile_size_px: u32, image_dimensions_px: (u32, u32)) -> Self {
        Self {
            tile_size_px,
            image_dimensions_px,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JigsawTile {
    pub index: JigsawIndex,
    pub in_place: bool,
}

#[derive(Debug, Serialize, Deserialize, Shrinkwrap)]
#[serde(transparent)]
pub struct PublicJigsawTile(Option<JigsawIndex>);

// #[derive(Debug, Serialize)]
// pub struct PublicJigsawPuzzle

impl From<JigsawTile> for PublicJigsawTile {
    fn from(value: JigsawTile) -> Self {
        Self(value.in_place.then_some(value.index))
    }
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct JigsawIndex {
    pub x: u32,
    pub y: u32,
}

impl From<JigsawIndex> for JigsawTile {
    fn from(value: JigsawIndex) -> Self {
        JigsawTile {
            index: value,
            in_place: false,
        }
    }
}

impl JigsawIndex {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }
}
