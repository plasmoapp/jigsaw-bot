use std::{path::PathBuf, sync::Arc};

use dptree::case;
use jigsaw_common::{
    model::request::generate_puzzle::GeneratePuzzleRequest, redis_scheme::RedisScheme,
};
use redis::aio::MultiplexedConnection;
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
        InlineQueryResultArticle,
        InputMessageContent, InputMessageContentText, Message, Update,
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

fn get_file_name(file: &teloxide::types::File) -> Option<String> {
    Some(PathBuf::from(&file.path).file_name()?.to_str()?.to_string())
}

async fn photo_handler(
    bot: Bot,
    message: Message,
    telegram_file: TelegramFile,
    config: Arc<Config>,
    mut redis_connection: MultiplexedConnection,
) -> Result<(), Report> {
    // Download a photo so generator can access it later

    let file_name = get_file_name(&telegram_file).expect("File name should alaways be valid");

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
    bot.send_message(message.chat.id, "Send an image to create a puzzle.")
        .await?;
    Ok(())
}

async fn start_handler(bot: Bot, message: Message) -> Result<(), Report> {
    bot.send_message(message.chat.id, "Welcome to the Jigsaw Puzzle bot!\n\nYou can turn any image into a jigsaw puzzle and then solve it together with friends without leaving Telegram.\n\nSend an image to create a puzzle.")
        .await?;
    Ok(())
}

async fn inline_handler(bot: Bot, query: InlineQuery, config: Arc<Config>) -> Result<(), Report> {
    let puzzle_uuid = Uuid::parse_str(&query.query)?;

    let button = InlineKeyboardButton::url("Play", config.get_puzzle_url(&puzzle_uuid));

    let image = config.get_puzzle_source_url(&puzzle_uuid);

    let article = InlineQueryResultArticle::new(
        "share".to_string(),
        "Share the puzzle",
        InputMessageContent::Text(InputMessageContentText::new(
            "Solve a Jigsaw Puzzle together!",
        )),
    )
    .thumb_url(image)
    .description("Click here to share")
    .reply_markup(InlineKeyboardMarkup::new([[button]]));

    let results = [InlineQueryResult::Article(article)];

    bot.answer_inline_query(&query.id, results).await?;

    respond(())?;

    Ok(())
}
