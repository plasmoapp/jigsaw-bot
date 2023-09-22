use std::sync::{Arc, Mutex};

use eyre::Report;
use jigsaw_common::model::{
    event::puzzle_generated::PuzzleGeneratedEvent,
    puzzle::{JigsawPuzzle, JigsawTile},
    request::generate_puzzle::GeneratePuzzleRequest,
};
use path_macro::path;
use redis::{aio::MultiplexedConnection, AsyncCommands, Msg};
use shrinkwraprs::Shrinkwrap;

use crate::{
    config::Config,
    jigsaw::RawJigsawPuzzle,
    storage::{JigsawFsImageStorage, JigsawImageStorage, JigsawRedisStateStorage, JigsawStorage},
};

pub struct JigsawGenerator {
    pub config: Config,
    pub storage: JigsawStorage,
}

impl JigsawGenerator {
    pub fn new(config: Config, storage: JigsawStorage) -> Self {
        // let redis_connection = redis::Client::open(config.redis_url.as_str())?
        //     .get_multiplexed_tokio_connection()
        //     .await?;

        // let storage = JigsawStorage::new(
        //     JigsawFsImageStorage::new(config.complete_storage_path.as_path()),
        //     JigsawRedisStateStorage {},
        // );

        Self { config, storage }
    }

    pub async fn generate_from_request(
        &self,
        request: &GeneratePuzzleRequest,
    ) -> Result<JigsawPuzzle, Report> {
        let image_path = path!(self.config.request_storage_path / request.image_path);
        let image = image::io::Reader::open(image_path)?.decode()?;

        let raw_puzzle = RawJigsawPuzzle::try_from_image(image)?;
        let puzzle = self.storage.store(raw_puzzle).await?;

        Ok(puzzle)
    }

    // // pub async fn handle_message(&mut self, msg: Msg) {
    //     // let result = self.generate_from_request(&request).await;

    //     let puzzle_uuid = match result {
    //         Ok(puzzle) => {
    //             debug!(
    //                 "Sucessfully generated puzzle {} for request {}",
    //                 puzzle.uuid, request.uuid
    //             );
    //             Some(puzzle.uuid)
    //         }
    //         Err(error) => {
    //             error!(
    //                 "Error when generating puzzle for request {}: {}",
    //                 request.uuid, error
    //             );
    //             None
    //         }
    //     };

    //     let event = PuzzleGeneratedEvent::new(request.uuid, puzzle_uuid);
    //     let Ok(message) = rmp_serde::to_vec(&event) else { return };

    //     let result = self
    //         .redis_connection
    //         .publish("event:puzzle_generated", message)
    //         .await;
    // }
}
