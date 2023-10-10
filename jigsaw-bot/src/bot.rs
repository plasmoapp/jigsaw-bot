use std::{ffi::OsStr, path::PathBuf, sync::Arc};

use dptree::case;
use jigsaw_common::{
    model::{puzzle::JigsawMeta, request::generate_puzzle::GeneratePuzzleRequest},
    redis_scheme::RedisScheme,
};
use redis::{aio::MultiplexedConnection, AsyncCommands};
use teloxide::{
    dispatching::{UpdateFilterExt, UpdateHandler},
    filter_command,
    macros::BotCommands,
    net::Download,
    payloads::SendMessageSetters,
    prelude::Dispatcher,
    requests::Requester,
    respond,
    types::{
        InlineKeyboardButton, InlineKeyboardMarkup, InlineQuery, InlineQueryResult,
        InlineQueryResultArticle, InlineQueryResultPhoto, InputMessageContent,
        InputMessageContentText, Message, Update,
    },
    Bot,
};
use uuid::Uuid;

use crate::config::Config;

use teloxide::types::File as TelegramFile;

use eyre::Report;

#[derive(Clone, BotCommands)]
#[command(rename_rule = "lowercase")]
pub enum Command {
    Start,
    Help,
}

pub async fn bot_main(bot: Bot, config: Arc<Config>, redis_connection: MultiplexedConnection) {
    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![config, redis_connection])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

fn schema() -> UpdateHandler<Report> {
    let command_branch = filter_command::<Command, _>()
        .branch(case![Command::Help].endpoint(help_handler))
        .branch(case![Command::Start].endpoint(start_handler));

    dptree::entry()
        .branch(
            Update::filter_message()
                .branch(dptree::filter_map_async(filter_photo).endpoint(photo_handler))
                .branch(command_branch)
                .endpoint(help_handler),
        )
        .branch(Update::filter_inline_query().endpoint(inline_handler))
}

// A function that filters out messages that have a photo and passes the photo to the dependency tree
// After that you can use File as an endpoint parameter
async fn filter_photo(message: Message, bot: Bot) -> Option<TelegramFile> {
    let photo = message.photo()?.last()?;
    let telegram_file = bot.get_file(&photo.file.id).await.ok()?;
    Some(telegram_file)
}

async fn photo_handler(
    bot: Bot,
    message: Message,
    telegram_file: TelegramFile,
    config: Arc<Config>,
    mut redis_connection: MultiplexedConnection,
) -> Result<(), Report> {
    // Download a photo so generator can access it later

    let file_name = PathBuf::from(&telegram_file.path)
        .file_name()
        .and_then(OsStr::to_str)
        .map(ToString::to_string)
        .expect("File name should alaways be valid");

    let mut file_path = config.request_storage_path.clone();

    file_path.push(&file_name);

    let mut fs_file = tokio::fs::File::create(file_path).await?;

    bot.download_file(&telegram_file.path, &mut fs_file).await?;

    // Send a request to the puzzle generator to generate a puzzle using redis

    let request = GeneratePuzzleRequest {
        uuid: Uuid::new_v4(),
        image_path: PathBuf::from(file_name),
    };

    // We also store a chat id in Redis so we can notify the user that the puzzle was generated
    // I choose not to put message id in the request to make it less coupled together
    // The generator module doesn't need to be aware of what is consuming its result

    let data_key = RedisScheme::request_message_data(&request.uuid);

    redis::pipe()
        .set(&data_key, rmp_serde::to_vec(&message.chat.id)?)
        .expire(&data_key, 60)
        .publish(
            RedisScheme::REQUEST_GENERATE_PUZZLE,
            rmp_serde::to_vec(&request)?,
        )
        .query_async(&mut redis_connection)
        .await?;

    // Notify user that the puzzle is being generated

    bot.send_message(message.chat.id, "Generating puzzle...")
        .reply_to_message_id(message.id)
        .await?;

    Ok(())
}

async fn help_handler(bot: Bot, message: Message) -> Result<(), Report> {
    bot.send_message(message.chat.id, "Send an image to create a puzzle 🧩")
        .await?;
    Ok(())
}

async fn start_handler(bot: Bot, message: Message) -> Result<(), Report> {
    bot.send_message(message.chat.id, "Welcome to the Jigsaw Puzzle bot!\n\nTurn any image into a Jigsaw Puzzle and solve it together with friends without leaving Telegram\n\nSend an image to create a puzzle 🧩")
        .await?;
    Ok(())
}

async fn inline_handler(
    bot: Bot,
    query: InlineQuery,
    config: Arc<Config>,
    mut redis: MultiplexedConnection,
) -> Result<(), Report> {
    let Ok(puzzle_uuid) = Uuid::parse_str(&query.query) else {
        bot.answer_inline_query(&query.id, []).await?;
        return Ok(());
    };

    let raw_meta = redis
        .get::<'_, _, Option<Vec<u8>>>(RedisScheme::jigsaw_puzzle_meta(&puzzle_uuid))
        .await?;

    let meta: JigsawMeta = match raw_meta {
        Some(raw_meta) => rmp_serde::from_slice(&raw_meta)?,
        None => {
            bot.answer_inline_query(&query.id, []).await?;
            return Ok(());
        }
    };

    let button = InlineKeyboardButton::url("Play", config.get_puzzle_url(&puzzle_uuid));

    let image = config.get_puzzle_preview_url(&puzzle_uuid);

    let article = InlineQueryResultArticle::new(
        "dummy".to_string(),
        "⬇️ ⬇️ ⬇️",
        InputMessageContent::Text(InputMessageContentText::new("Jigsaw Puzzle Bot by @kpids")),
    );

    let photo = InlineQueryResultPhoto::new("share", image.clone(), image.clone())
        .photo_width(meta.image_dimensions_px.0 as i32)
        .photo_height(meta.image_dimensions_px.1 as i32)
        .title("Share puzzle")
        .description("Click here to share")
        .caption("Solve a Jigsaw Puzzle together!")
        .reply_markup(InlineKeyboardMarkup::new([[button]]));

    let results = [
        InlineQueryResult::Article(article),
        InlineQueryResult::Photo(photo),
    ];

    bot.answer_inline_query(&query.id, results).await?;

    respond(())?;

    Ok(())
}
