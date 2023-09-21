use std::{path::PathBuf};

use async_trait::async_trait;
use eyre::Report;
use futures::{stream, StreamExt, TryStreamExt};
use image::ImageFormat;
use path_macro::path;

use crate::jigsaw::{IndexedRawJigsawPuzzle, JigsawPuzzle, RawJigsawPuzzle};

pub struct JigsawStorage {
    image_storage: Box<dyn JigsawImageStorage>,
    state_storage: Box<dyn JigsawStateStorage>,
}

impl JigsawStorage {
    pub fn new(
        image_storage: impl JigsawImageStorage + 'static,
        state_storage: impl JigsawStateStorage + 'static,
    ) -> Self {
        Self {
            image_storage: Box::new(image_storage),
            state_storage: Box::new(state_storage),
        }
    }

    pub async fn store(&self, puzzle: RawJigsawPuzzle) -> Result<JigsawPuzzle, Report> {
        let indexed: IndexedRawJigsawPuzzle = puzzle.into();
        self.image_storage.store(&indexed).await?;
        let puzzle = self.state_storage.store(indexed).await?;
        Ok(puzzle)
    }
}

#[async_trait]
pub trait JigsawImageStorage {
    async fn store(&self, puzzle: &IndexedRawJigsawPuzzle) -> Result<(), Report>;
}

#[async_trait]
pub trait JigsawStateStorage {
    async fn store(&self, puzzle: IndexedRawJigsawPuzzle) -> Result<JigsawPuzzle, Report>;
}

pub struct JigsawFsImageStorage {
    path: PathBuf,
}

impl JigsawFsImageStorage {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

#[async_trait]
impl JigsawImageStorage for JigsawFsImageStorage {
    async fn store(&self, puzzle: &IndexedRawJigsawPuzzle) -> Result<(), Report> {
        let puzzle_dir = path!(self.path / puzzle.puzzle_source.id.to_string());

        tokio::fs::create_dir_all(&puzzle_dir).await?;

        let puzzle_source_path = path!(&puzzle_dir / "source.jpeg");

        let puzzle_dir_ref = &puzzle_dir;

        puzzle
            .puzzle_source
            .save_with_format(puzzle_source_path, ImageFormat::Jpeg)?;

        stream::iter(&puzzle.tile_vec)
            .then(|tile| async move {
                let tile_path = path!(&puzzle_dir_ref / format!("{}.jpeg", tile.id.to_string()));
                tile.image.save_with_format(tile_path, ImageFormat::Jpeg)?;
                Ok::<(), Report>(())
            })
            .try_collect::<Vec<_>>()
            .await?;

        Ok(())
    }
}

pub struct JigsawRedisStateStorage {}

#[async_trait]
impl JigsawStateStorage for JigsawRedisStateStorage {
    async fn store(&self, puzzle: IndexedRawJigsawPuzzle) -> Result<JigsawPuzzle, Report> {
        let puzzle: JigsawPuzzle = puzzle.into();

        // TODO

        Ok(puzzle)
    }
}
