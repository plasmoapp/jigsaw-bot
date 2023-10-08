use eyre::Report;
use image::{imageops::FilterType, DynamicImage, GenericImage, GenericImageView, RgbaImage};
use rayon::prelude::*;
use tiny_skia::{IntSize, Pixmap};
use uuid::Uuid;

use jigsaw_common::{
    model::puzzle::{JigsawIndex, JigsawMeta, JigsawPuzzle, JigsawTile},
    util::indexed::Indexed,
};

use crate::jigsaw_connections::PieceConnections;

const BIGGER_SIDE_SIZE_PX: u32 = 1920;

const TILES_PER_BIGGER_SIDE: u32 = 8;

const TILE_SIZE_PX: u32 = BIGGER_SIDE_SIZE_PX / TILES_PER_BIGGER_SIDE;
const TILE_PADDING_PX: u32 = TILE_SIZE_PX / 2;
const TILE_WITH_PADDING_SIZE_PX: u32 = TILE_SIZE_PX * 2;

pub struct RawJigsawPuzzle {
    pub puzzle_source: DynamicImage,
    pub tile_vec: Vec<RawJigsawTile>,
    pub meta: JigsawMeta,
}

pub struct IndexedRawJigsawPuzzle {
    pub puzzle_source: Indexed<Uuid, DynamicImage>,
    pub tile_vec: Vec<Indexed<Uuid, RawJigsawTile>>,
    pub meta: JigsawMeta,
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
            meta: value.meta,
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
            value.meta,
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

        let connections = PieceConnections::generate_connections_for_size(
            new_width,
            new_height,
            TILE_SIZE_PX,
            None,
        );

        let tiles_x = (new_width / TILE_SIZE_PX) as usize;
        let tiles_y = (new_height / TILE_SIZE_PX) as usize;

        let tile_vec = (0..new_width)
            .into_par_iter()
            .step_by(TILE_SIZE_PX as usize)
            .flat_map(|corner_x| {
                (0..new_height)
                    .into_par_iter()
                    .step_by(TILE_SIZE_PX as usize)
                    .map(|corner_y| {
                        let x = (corner_x / TILE_SIZE_PX) as usize;
                        let y = (corner_y / TILE_SIZE_PX) as usize;

                        let (crop_x, crop_width) = {
                            if x == 0 {
                                if tiles_x > 1 {
                                    (0, TILE_SIZE_PX + TILE_PADDING_PX)
                                } else {
                                    (0, TILE_SIZE_PX)
                                }
                            } else if x == tiles_x - 1 {
                                (TILE_PADDING_PX, TILE_PADDING_PX + TILE_SIZE_PX)
                            } else {
                                (
                                    TILE_PADDING_PX,
                                    TILE_PADDING_PX + TILE_SIZE_PX + TILE_PADDING_PX,
                                )
                            }
                        };
                        let (crop_y, crop_height) = {
                            if y == 0 {
                                if tiles_y > 1 {
                                    (0, TILE_SIZE_PX + TILE_PADDING_PX)
                                } else {
                                    (0, TILE_SIZE_PX)
                                }
                            } else if y == tiles_y - 1 {
                                (TILE_PADDING_PX, TILE_PADDING_PX + TILE_SIZE_PX)
                            } else {
                                (
                                    TILE_PADDING_PX,
                                    TILE_PADDING_PX + TILE_SIZE_PX + TILE_PADDING_PX,
                                )
                            }
                        };

                        let connection = connections[y][x].clone();

                        let index =
                            JigsawIndex::new(corner_x / TILE_SIZE_PX, corner_y / TILE_SIZE_PX);
                        let crop = new_image.crop_imm(
                            corner_x - crop_x,
                            corner_y - crop_y,
                            crop_width,
                            crop_height,
                        );
                        let mut container_image = DynamicImage::new_rgba8(
                            TILE_WITH_PADDING_SIZE_PX,
                            TILE_WITH_PADDING_SIZE_PX,
                        );

                        container_image
                            .copy_from(&crop, TILE_PADDING_PX - crop_x, TILE_PADDING_PX - crop_y)
                            .unwrap();
                        let image_bytes = container_image.to_rgba8().to_vec();

                        let mask = connection.create_piece_mask(
                            TILE_WITH_PADDING_SIZE_PX,
                            TILE_WITH_PADDING_SIZE_PX,
                            TILE_SIZE_PX,
                        );

                        let mut pixmap = Pixmap::from_vec(
                            image_bytes,
                            IntSize::from_wh(TILE_WITH_PADDING_SIZE_PX, TILE_WITH_PADDING_SIZE_PX)
                                .unwrap(),
                        )
                        .unwrap();

                        pixmap.apply_mask(&mask);

                        let final_image = DynamicImage::from(
                            RgbaImage::from_raw(
                                TILE_WITH_PADDING_SIZE_PX,
                                TILE_WITH_PADDING_SIZE_PX,
                                pixmap.data().to_vec(),
                            )
                            .unwrap(),
                        );

                        RawJigsawTile::new(index, final_image)
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        Ok(RawJigsawPuzzle::new(
            new_image,
            tile_vec,
            JigsawMeta::new(TILE_SIZE_PX, (new_width, new_height)),
        ))
    }

    pub fn new(
        puzzle_source: DynamicImage,
        tile_vec: Vec<RawJigsawTile>,
        meta: JigsawMeta,
    ) -> Self {
        Self {
            puzzle_source,
            tile_vec,
            meta,
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
