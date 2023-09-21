pub mod config;
pub mod jigsaw;
pub mod storage;

use std::sync::Arc;

use eyre::Report;
use jigsaw::RawJigsawPuzzle;
use storage::{JigsawFsImageStorage, JigsawRedisStateStorage, JigsawStorage};

// use crate::jigsaw::{JigsawInitizlizer, LocalInitializer, RawJigsawPuzzle};

#[tokio::main]
async fn main() -> Result<(), Report> {
    dotenvy::dotenv()?;

    let storage = Arc::new(JigsawStorage::new(
        JigsawFsImageStorage::new("data"),
        JigsawRedisStateStorage {},
    ));

    let image = image::io::Reader::open("test.jpg")?.decode()?;

    let raw_puzzle = RawJigsawPuzzle::try_from_image(image)?;

    let puzzle = storage.store(raw_puzzle).await?;

    dbg!(puzzle);

    Ok(())
}
