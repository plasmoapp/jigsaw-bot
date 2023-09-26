pub mod bot;
pub mod config;
pub mod sub;

use std::sync::Arc;

use config::Config;
use jigsaw_common::{
    model::request::generate_puzzle::GeneratePuzzleRequest, util::config::default_extract_config,
};

use eyre::Report;
use teloxide::Bot;

use crate::{bot::bot_main, sub::pubsub_main};

#[tokio::main]
async fn main() -> Result<(), Report> {
    dotenvy::dotenv()?;
    env_logger::init();

    let config = Arc::new(default_extract_config::<Config>()?);

    let redis_connection = redis::Client::open(config.redis_url.as_str())?
        .get_multiplexed_tokio_connection()
        .await?;

    let bot = Bot::from_env();

    let mut redis_pubsub = redis::Client::open(config.redis_url.as_str())?
        .get_tokio_connection()
        .await?
        .into_pubsub();

    redis_pubsub.subscribe("event:puzzle_generated").await?;

    tokio::select! {
        _ = tokio::task::spawn(bot_main(bot.clone(), config.clone(), redis_connection.clone())) => {},
        _ = tokio::task::spawn(pubsub_main(redis_pubsub, bot.clone(), config.clone(), redis_connection.clone())) => {},
    }

    Ok(())
}
