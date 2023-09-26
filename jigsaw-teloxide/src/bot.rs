use std::{path::PathBuf, sync::Arc};

use jigsaw_common::model::request::generate_puzzle::GeneratePuzzleRequest;
use redis::aio::MultiplexedConnection;
use teloxide::{
    dispatching::{UpdateFilterExt, UpdateHandler},
    net::Download,
    payloads::SendMessageSetters,
    prelude::Dispatcher,
    requests::Requester,
    respond,
    types::{
        CallbackQuery, InlineKeyboardButton, InlineKeyboardMarkup, InlineQuery, InlineQueryResult,
        InlineQueryResultArticle, InputMessageContent, InputMessageContentText, Message, PhotoSize,
        Update, WebAppInfo,
    },
    Bot,
};
use uuid::Uuid;

use crate::config::Config;

use eyre::Report;

pub async fn bot_main(bot: Bot, config: Arc<Config>, redis_connection: MultiplexedConnection) {
    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![config, redis_connection])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

fn schema() -> UpdateHandler<Report> {
    dptree::entry().branch(Update::filter_message().endpoint(message_handler))
    // .branch(Update::filter_inline_query().endpoint(inline_handler))
    // .branch(Update::filter_callback_query().endpoint(callback_handler))
}

fn get_first_photo(message: &Message) -> Option<&PhotoSize> {
    message.photo()?.last()
}

fn get_file_name(file: &teloxide::types::File) -> Option<String> {
    Some(PathBuf::from(&file.path).file_name()?.to_str()?.to_string())
}

async fn message_handler(
    bot: Bot,
    message: Message,
    config: Arc<Config>,
    mut redis_connection: MultiplexedConnection,
) -> Result<(), Report> {
    let Some(photo) = get_first_photo(&message) else {
        return Ok(())
    };

    let telegram_file = bot.get_file(&photo.file.id).await?;

    let Some(file_name) = get_file_name(&telegram_file) else {
        return Ok(())
    };

    let mut file_path = config.request_storage_path.clone();
    file_path.push(&file_name);

    let mut fs_file = tokio::fs::File::create(file_path).await?;

    bot.download_file(&telegram_file.path, &mut fs_file).await?;

    let request = GeneratePuzzleRequest {
        uuid: Uuid::new_v4(),
        image_path: PathBuf::from(file_name),
    };

    let data_key = format!("request_message_data:{}", request.uuid);

    redis::pipe()
        .set(&data_key, rmp_serde::to_vec(&message.chat.id)?)
        .expire(&data_key, 60)
        .publish("request:generate_puzzle", rmp_serde::to_vec(&request)?)
        .query_async(&mut redis_connection)
        .await?;

    bot.send_message(message.chat.id, "Generating puzzle...")
        .reply_to_message_id(message.id)
        .await?;

    Ok(())
}

// async fn inline_handler(bot: Bot, query: InlineQuery, config: Arc<Config>) -> Result<(), Report> {
//     dbg!(&query);

//     let puzzle_uuid = Uuid::parse_str(&query.query)?;

//     let mut url = config.web_app_url.clone();

//     url.query_pairs_mut()
//         .append_pair("puzzle", &puzzle_uuid.to_string());

//     let button = InlineKeyboardButton::web_app("Play", WebAppInfo { url });

//     let article = InlineQueryResultArticle::new(
//         "share".to_string(),
//         "Share puzzle",
//         InputMessageContent::Text(InputMessageContentText::new("Jigsaw Puzzle")),
//     )
//     .reply_markup(InlineKeyboardMarkup::new([[button]]));

//     let results = vec![InlineQueryResultButton];

//     bot.answer_inline_query(&query.id, results).await?;

//     respond(())?;

//     Ok(())
// }

// pub async fn callback_handler(
//     bot: Bot,
//     query: CallbackQuery,
//     // storage: Arc<ConcurrentHashMap<String, Game>>,
// ) -> Result<(), Report> {
//     dbg!(query);
// let state = storage
//     .get_or_default(q.inline_message_id.as_ref().unwrap())
//     .await;

// let mut lock = state.lock().await;

// match lock.process_callback(q.clone()) {
//     Ok(_) => update_message(bot, q, &lock).await?,
//     Err(error) => {
//         bot.answer_callback_query(q.id)
//             .text(error.to_string())
//             .show_alert(true)
//             .await?;
//     }
// };

//     Ok(())
// }
