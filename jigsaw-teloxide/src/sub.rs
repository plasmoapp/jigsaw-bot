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
                    error!("Received invalid request: {error}");
                    return;
                }
            };

            if let Err(error) = process_event(event, bot, config, redis_connection).await {
                error!("Error while processing event: {error}");
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

    let mut url = config.web_app_url.clone();

    url.query_pairs_mut()
        .append_pair("puzzle", &event.puzzle_uuid.unwrap().to_string());

    // dbg!(&url);

    // TODO: Handle Error
    // url.path_segments_mut()
    //     .unwrap()
    //     .push("puzzle")
    //     .push(&event.puzzle_uuid.unwrap().to_string());

    let play_button = InlineKeyboardButton::web_app("Open", WebAppInfo { url });
    let share_button: InlineKeyboardButton =
        InlineKeyboardButton::switch_inline_query("Share", event.puzzle_uuid.unwrap().to_string());

    let keyboard = InlineKeyboardMarkup::new([[play_button], [share_button]]);

    bot.send_message(message, "Puzzle generated succesfully")
        .reply_markup(keyboard)
        .await?;

    Ok(())
}
