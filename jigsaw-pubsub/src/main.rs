#[macro_use]
extern crate log;

pub mod config;
pub mod generator;
pub mod jigsaw;
pub mod storage;

use std::{path::PathBuf, sync::Arc};

use eyre::Report;
use futures::StreamExt;
use generator::JigsawGenerator;
use jigsaw::RawJigsawPuzzle;
use jigsaw_common::{
    model::{
        event::puzzle_generated::PuzzleGeneratedEvent, puzzle::JigsawPuzzle,
        request::generate_puzzle::GeneratePuzzleRequest,
    },
    util::config::default_extract_config,
};
use redis::{aio::MultiplexedConnection, AsyncCommands, Msg};
use storage::{JigsawFsImageStorage, JigsawRedisStateStorage, JigsawStorage};
use tokio::task;
use uuid::Uuid;

use crate::config::Config;

// use crate::jigsaw::{JigsawInitizlizer, LocalInitializer, RawJigsawPuzzle};

#[tokio::main]
async fn main() -> Result<(), Report> {
    dotenvy::dotenv()?;

    env_logger::init();

    let config = default_extract_config::<Config>()?;

    let mut redis_pubsub = redis::Client::open(config.redis_url.as_str())?
        .get_tokio_connection()
        .await?
        .into_pubsub();

    redis_pubsub.subscribe("request:generate_puzzle").await?;

    let redis_connection = redis::Client::open(config.redis_url.as_str())?
        .get_multiplexed_tokio_connection()
        .await?;

    let storage = JigsawStorage::new(
        JigsawFsImageStorage::new(config.complete_storage_path.as_path()),
        JigsawRedisStateStorage::new(redis_connection.clone()),
    );

    let jigsaw_generator = Arc::new(JigsawGenerator::new(config, storage));

    info!("Service launched");

    while let Some(msg) = redis_pubsub.on_message().next().await {
        let jigsaw_generator = jigsaw_generator.clone();
        let mut redis_connection = redis_connection.clone();

        tokio::task::spawn(async move {
            let request_result =
                rmp_serde::from_slice::<GeneratePuzzleRequest>(msg.get_payload_bytes());

            let request = match request_result {
                Ok(request) => request,
                Err(error) => {
                    error!("Received invalid request: {error}");
                    return;
                }
            };

            let generate_result = jigsaw_generator.generate_from_request(&request).await;

            let puzzle_uuid = match generate_result {
                Ok(puzzle) => {
                    debug!(
                        "Sucessfully generated puzzle {} for request {}",
                        puzzle.uuid, request.uuid
                    );
                    Some(puzzle.uuid)
                }
                Err(error) => {
                    error!(
                        "Error when generating puzzle for request {}: {}",
                        request.uuid, error
                    );
                    None
                }
            };

            let event = PuzzleGeneratedEvent::new(request.uuid, puzzle_uuid);
            let Ok(message) = rmp_serde::to_vec(&event) else { return };

            let result: Result<(), _> = redis_connection
                .publish("event:puzzle_generated", message)
                .await;

            if let Err(error) = result {
                error!("Error while sending event: {error}");
            }
        });
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use jigsaw_common::model::request::generate_puzzle::GeneratePuzzleRequest;
    use redis::AsyncCommands;
    use uuid::Uuid;

    use crate::config::Config;

    use eyre::Report;

    #[tokio::test]
    async fn send_request() -> Result<(), Report> {
        dotenvy::dotenv()?;

        env_logger::init();

        let config = Config::extract()?;

        let mut redis = redis::Client::open(config.redis_url.as_str())?
            .get_tokio_connection()
            .await?;

        let message = GeneratePuzzleRequest {
            uuid: Uuid::new_v4(),
            image_path: PathBuf::from("test.jpeg"),
        };

        redis
            .publish("request:generate_puzzle", rmp_serde::to_vec(&message)?)
            .await?;

        Ok(())
    }
}
