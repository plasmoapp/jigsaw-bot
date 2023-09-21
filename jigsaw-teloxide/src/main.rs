use teloxide::{requests::Requester, types::Message, Bot};

use eyre::Report;

#[tokio::main]
async fn main() -> Result<(), Report> {
    dotenvy::dotenv()?;

    let bot = Bot::from_env();

    teloxide::repl(bot, |bot: Bot, msg: Message| async move {
        bot.send_dice(msg.chat.id).await?;
        Ok(())
    })
    .await;

    Ok(())
}
