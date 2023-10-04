use std::sync::Arc;

use eyre::Report;
use futures::StreamExt;
use jigsaw_common::model::event::puzzle_generated::PuzzleGeneratedEvent;
use log::error;
use redis::{
    aio::{MultiplexedConnection, PubSub},
    AsyncCommands,
};
use teloxide::{
    payloads::SendMessageSetters,
    requests::Requester,
    types::{
        ChatId, InlineKeyboardButton, InlineKeyboardButtonKind, InlineKeyboardMarkup, Message,
        WebAppInfo,
    },
    Bot,
};
use url::Url;

use crate::config::Config;

pub async fn pubsub_main(
    mut redis_pubsub: PubSub,
    bot: Bot,
    config: Arc<Config>,
    redis_connection: MultiplexedConnection,
) {
    while let Some(msg) = redis_pubsub.on_message().next().await {
        let redis_connection = redis_connection.clone();
        let bot = bot.clone();
        let config = config.clone();

        tokio::task::spawn(async move {
            let event_result =
                rmp_serde::from_slice::<PuzzleGeneratedEvent>(msg.get_payload_bytes());

            let event = match event_result {
                Ok(request) => request,
                Err(error) => {
                    error!("Received an invalid message: {error}");
                    return;
                }
            };

            if let Err(error) = process_event(event, bot, config, redis_connection).await {
                error!("Error while processing an event: {error}");
            }
        });
    }
}

pub async fn process_event(
    event: PuzzleGeneratedEvent,
    bot: Bot,
    config: Arc<Config>,
    mut redis_connection: MultiplexedConnection,
) -> Result<(), Report> {
    let data_key = format!("request_message_data:{}", event.request_uuid);
    let message_raw = redis_connection.get::<_, Vec<u8>>(data_key).await?;
    let message = rmp_serde::from_slice::<ChatId>(&message_raw)?;

    let puzzle_uuid = match event.puzzle_uuid {
        Some(puzzle_uuid) => puzzle_uuid,
        None => {
            bot.send_message(message, "Failed to create a puzzle. It's likely that the image is too small. Try a different one!").await?;
            return Ok(());
        }
    };

    let play_button = InlineKeyboardButton::url("Play alone", config.get_puzzle_url(&puzzle_uuid));
    let share_button: InlineKeyboardButton =
        InlineKeyboardButton::switch_inline_query("Share with friends", puzzle_uuid.to_string());

    let keyboard = InlineKeyboardMarkup::new([[share_button], [play_button]]);

    bot.send_message(message, "Done! Now share it with friends or play alone")
        .reply_markup(keyboard)
        .await?;

    Ok(())
}
