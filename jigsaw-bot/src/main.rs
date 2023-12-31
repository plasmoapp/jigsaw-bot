pub mod bot;
pub mod config;
pub mod pubsub;

use std::sync::Arc;

use config::Config;
use jigsaw_common::{redis_scheme::RedisScheme, util::config::default_extract_config};

use eyre::Report;
use teloxide::Bot;

use crate::{bot::bot_main, pubsub::pubsub_main};

#[tokio::main]
async fn main() -> Result<(), Report> {
    _ = dotenvy::dotenv();
    env_logger::init();

    let config = Arc::new(default_extract_config::<Config>()?);

    let redis_connection = redis::Client::open(config.redis_url.as_str())?
        .get_multiplexed_tokio_connection()
        .await?;

    let bot = Bot::new(&config.bot_token);

    let mut redis_pubsub = redis::Client::open(config.redis_url.as_str())?
        .get_tokio_connection()
        .await?
        .into_pubsub();

    redis_pubsub
        .subscribe(RedisScheme::EVENT_PUZZLE_GENERATED)
        .await?;

    tokio::select! {
        _ = tokio::task::spawn(bot_main(bot.clone(), config.clone(), redis_connection.clone())) => {},
        _ = tokio::task::spawn(pubsub_main(redis_pubsub, bot.clone(), config.clone(), redis_connection.clone())) => {},
    }

    Ok(())
}
