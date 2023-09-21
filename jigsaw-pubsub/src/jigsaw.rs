use std::{collections::HashMap};

use eyre::Report;
use futures::{StreamExt};
use image::{
    imageops::FilterType::{self},
    DynamicImage, GenericImageView,
};
use jigsaw_common::util::indexed::Indexed;







use rayon::prelude::*;
use uuid::Uuid;

const BIGGER_SIDE_SIZE_PX: u32 = 1920;

const TILES_PER_BIGGER_SIDE: u32 = 8;

const TILE_SIZE_PX: u32 = BIGGER_SIDE_SIZE_PX / TILES_PER_BIGGER_SIDE;

#[derive(Debug)]
pub struct JigsawPuzzle {
    uuid: Uuid,
    tile_map: HashMap<Uuid, JigsawTile>,
}

impl JigsawPuzzle {
    pub fn new(uuid: Uuid, tile_map: HashMap<Uuid, JigsawTile>) -> Self {
        Self { uuid, tile_map }
    }
}

#[derive(Debug)]
pub struct JigsawTile {
    pub index: JigsawIndex,
    pub in_place: bool,
}

#[derive(PartialEq, Eq, Debug)]
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

pub struct RawJigsawPuzzle {
    pub puzzle_source: DynamicImage,
    pub tile_vec: Vec<RawJigsawTile>,
}

pub struct IndexedRawJigsawPuzzle {
    pub puzzle_source: Indexed<Uuid, DynamicImage>,
    pub tile_vec: Vec<Indexed<Uuid, RawJigsawTile>>,
}

impl From<RawJigsawPuzzle> for IndexedRawJigsawPuzzle {
    fn from(value: RawJigsawPuzzle) -> Self {
        IndexedRawJigsawPuzzle {
            puzzle_source: value.puzzle_source.into(),
            tile_vec: value
                .tile_vec
                .into_iter()
                .map(|value| value.into())
                .collect(),
        }
    }
}

impl From<IndexedRawJigsawPuzzle> for JigsawPuzzle {
    fn from(value: IndexedRawJigsawPuzzle) -> Self {
        JigsawPuzzle::new(
            value.puzzle_source.id,
            value
                .tile_vec
                .into_iter()
                .map(|tile| (tile.id, JigsawTile::from(tile.value.index)))
                .collect(),
        )
    }
}

impl RawJigsawPuzzle {
    /// Returns None if image can't be made into a puzzle because of a bad aspect ratio
    fn try_get_puzzle_image_dimensions(image: &DynamicImage) -> Option<(u32, u32)> {
        let (width, height) = image.dimensions();

        let is_landscape = width >= height;

        let (bigger, smaller) = if is_landscape {
            (width, height)
        } else {
            (height, width)
        };

        let image_tile_size_px = bigger as f32 / TILES_PER_BIGGER_SIDE as f32;

        let tiles_fit_on_smaller = (smaller as f32 / image_tile_size_px) as u32;

        if tiles_fit_on_smaller == 0 {
            return None;
        }

        let new_smaller = tiles_fit_on_smaller * TILE_SIZE_PX;

        let result = if is_landscape {
            (BIGGER_SIDE_SIZE_PX, new_smaller)
        } else {
            (new_smaller, BIGGER_SIDE_SIZE_PX)
        };

        Some(result)
    }

    pub fn try_from_image(image: DynamicImage) -> Result<Self, Report> {
        let (new_width, new_height) = Self::try_get_puzzle_image_dimensions(&image).unwrap();

        let new_image = image.resize_to_fill(new_width, new_height, FilterType::Lanczos3);

        let tile_vec = (0..new_width)
            .into_par_iter()
            .step_by(TILE_SIZE_PX as usize)
            .flat_map(|corner_x| {
                (0..new_height)
                    .into_par_iter()
                    .step_by(TILE_SIZE_PX as usize)
                    .map(|corner_y| {
                        let index =
                            JigsawIndex::new(corner_x / TILE_SIZE_PX, corner_y / TILE_SIZE_PX);
                        let image =
                            new_image.crop_imm(corner_x, corner_y, TILE_SIZE_PX, TILE_SIZE_PX);
                        RawJigsawTile::new(index, image)
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        Ok(RawJigsawPuzzle::new(new_image, tile_vec))
    }

    pub fn new(puzzle_source: DynamicImage, tile_vec: Vec<RawJigsawTile>) -> Self {
        Self {
            puzzle_source,
            tile_vec,
        }
    }
}

pub struct RawJigsawTile {
    pub image: DynamicImage,
    pub index: JigsawIndex,
}

impl RawJigsawTile {
    pub fn new(index: JigsawIndex, image: DynamicImage) -> Self {
        Self { index, image }
    }
}
