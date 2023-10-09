use std::path::PathBuf;

use async_trait::async_trait;
use eyre::Report;
use futures::{stream, StreamExt, TryStreamExt};
use image::ImageFormat;
use jigsaw_common::{model::puzzle::JigsawPuzzle, redis_scheme::RedisScheme};
use path_macro::path;
use redis::{aio::MultiplexedConnection, AsyncCommands};

use crate::jigsaw::{IndexedRawJigsawPuzzle, RawJigsawPuzzle};

pub struct JigsawStorage<I, S>
where
    I: JigsawImageStorage + Sync + Send,
    S: JigsawStateStorage + Sync + Send,
{
    image_storage: I,
    state_storage: S,
}

impl<I, S> JigsawStorage<I, S>
where
    I: JigsawImageStorage + Sync + Send,
    S: JigsawStateStorage + Sync + Send,
{
    pub fn new(image_storage: I, state_storage: S) -> Self {
        Self {
            image_storage,
            state_storage,
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

        let puzzle_source_path = path!(&puzzle_dir / "source.webp");

        let puzzle_preview_path = path!(&puzzle_dir / "preview.jpeg");

        let puzzle_dir_ref = &puzzle_dir;

        puzzle
            .puzzle_source
            .save_with_format(puzzle_source_path, ImageFormat::WebP)?;

        puzzle
            .puzzle_source
            .save_with_format(puzzle_preview_path, ImageFormat::Jpeg)?;

        stream::iter(&puzzle.tile_vec)
            .then(|tile| async move {
                let tile_path = path!(&puzzle_dir_ref / format!("{}.webp", tile.id.to_string()));
                tile.image.save_with_format(tile_path, ImageFormat::WebP)?;
                Ok::<(), Report>(())
            })
            .try_collect::<Vec<_>>()
            .await?;

        Ok(())
    }
}

pub struct JigsawRedisStateStorage {
    redis_connection: MultiplexedConnection,
}

impl JigsawRedisStateStorage {
    pub fn new(redis_connection: MultiplexedConnection) -> Self {
        Self { redis_connection }
    }
}

#[async_trait]
impl JigsawStateStorage for JigsawRedisStateStorage {
    async fn store(&self, puzzle: IndexedRawJigsawPuzzle) -> Result<JigsawPuzzle, Report> {
        let puzzle: JigsawPuzzle = puzzle.into();

        let mut redis_connection = self.redis_connection.clone();

        let state_key = RedisScheme::jigsaw_puzzle_state(&puzzle.uuid);

        let state_map = puzzle
            .tile_map
            .iter()
            .map(|(key, value)| Ok((key.to_string(), rmp_serde::to_vec(&value)?)))
            .collect::<Result<Vec<(_, _)>, Report>>()?;

        let meta_key = RedisScheme::jigsaw_puzzle_meta(&puzzle.uuid);

        let meta = rmp_serde::to_vec(&puzzle.meta)?;

        redis::pipe()
            .hset_multiple(state_key, &state_map)
            .set(meta_key, meta)
            .query_async(&mut redis_connection)
            .await?;

        Ok(puzzle)
    }
}
