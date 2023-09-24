use teloxide::{
    dispatching::{dialogue, dialogue::InMemStorage, UpdateHandler},
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
    utils::command::BotCommands,
};
use url::Url;

type MyDialogue = Dialogue<State, InMemStorage<State>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[derive(Clone, Default)]
pub enum State {
    #[default]
    Start,
    ReceiveLang,
    Lang(Lang)
}

#[derive(Debug,Clone)]
pub enum Lang {
    En,
    Ar,
    Es,
    Fr
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "start the purchase procedure.")]
    Start,
    #[command(description = "reset your lang")]
    Reset
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting purchase bot...");

    let bot = Bot::new("6303406200:AAFQvEFi91G_BY6MpmOIz2YawrngUecZ4L8");

    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![InMemStorage::<State>::new()])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

fn schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    use dptree::case;

    let command_handler = teloxide::filter_command::<Command, _>()
        .branch(
            case![State::Start]
                .branch(case![Command::Start].endpoint(start)),
        )
        .branch(case![Command::Help].endpoint(help))
        .branch(case![Command::Reset].endpoint(reset));

    let message_handler = Update::filter_message()
        .branch(command_handler)
        .branch(case![State::Lang ( l )].endpoint(parse_url))
        .branch(dptree::endpoint(invalid_state));

    let callback_query_handler = Update::filter_callback_query().branch(
        case![State::ReceiveLang].endpoint(receive_lang),
    );

    dialogue::enter::<Update, InMemStorage<State>, State, _>()
        .branch(message_handler)
        .branch(callback_query_handler)
}

async fn reset(bot: Bot, dialogue: MyDialogue) -> HandlerResult {
    dialogue.exit().await?;
    Ok(())
}

async fn start(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    let langs = ["Ar", "En", "Es", "Fr"]
        .map(|lang| InlineKeyboardButton::callback(lang, lang));

    bot.send_message(msg.chat.id, "Select a lang:")
        .reply_markup(InlineKeyboardMarkup::new([langs]))
        .await?;

    dialogue.update(State::ReceiveLang).await?;
    Ok(())
}

async fn help(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?;
    Ok(())
}

async fn invalid_state(bot: Bot, msg: Message) -> HandlerResult {
    parse_url(bot, msg, Lang::En).await
}

async fn receive_lang(
    bot: Bot,
    dialogue: MyDialogue,
    q: CallbackQuery,
) -> HandlerResult {
    if let Some(lang) = &q.data {
        bot.answer_callback_query(q.id).await?;

        bot.send_message(
            dialogue.chat_id(),
            format!(" product '{lang}' has been purchased successfully!"),
        )
        .await?;
        match lang.as_str() {
            "Ar" => dialogue.update(State::Lang(Lang::Ar)).await?,
            "En" => dialogue.update(State::Lang(Lang::En)).await?,
            "Fr" => dialogue.update(State::Lang(Lang::Fr)).await?,
            "Es" => dialogue.update(State::Lang(Lang::Es)).await?,
            _    => return Ok(()),
        }
//        dialogue.exit().await?;
    }

    Ok(())
}

async fn parse_url(bot: Bot, msg: Message, lang: Lang) -> HandlerResult {
    match Url::parse(msg.text().unwrap()) {
        Ok(url) => {
            match url.host_str() {
                Some("youtube.com") | Some("youtu.be") =>
                    bot.send_message(msg.chat.id,format!("youtube"))
                    .await?,
                Some(_) => bot.send_message(msg.chat.id,format!("not supported"))
                    .await?,
                None => bot.send_message(msg.chat.id,format!("WTF"))
                    .await?,
            };
        }
        Err(e) => {
            bot.send_message(msg.chat.id,format!("ERROR: {}", e)).await?;
        },
    };
    Ok(())
}
