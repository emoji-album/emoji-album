use dotenv::dotenv;
use rand::rngs::StdRng;
use rand::seq::IteratorRandom;
use rand::FromEntropy;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs;
use std::string::ToString;
use std::sync::{Arc, Mutex};
use teloxide::{prelude::*, utils::command::BotCommand};

#[derive(BotCommand)]
#[command(rename = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "rolls 5 random emojis.")]
    Roll,
    #[command(description = "handle a username and an age.", parse_with = "split")]
    UsernameAndAge { username: String, age: u8 },
}

type Username = String;

lazy_static::lazy_static! {
    static ref USER_EMOJIS: Arc<Mutex<HashMap<Username, Vec<char>>>> = Arc::new(Mutex::new(HashMap::new()));
    static ref EMOJI_FILE: String = fs::read_to_string("emojis.csv").unwrap();
    static ref EMOJIS: Vec<&'static str> = EMOJI_FILE.trim().split('\n').collect();
}

async fn answer(
    cx: UpdateWithCx<AutoSend<Bot>, Message>,
    command: Command,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    match command {
        Command::Help => cx.answer(Command::descriptions()).send().await?,
        Command::Roll => {
            let mut rng = StdRng::from_entropy();

            let random_emojis: Vec<String> = EMOJIS
                .iter()
                .choose_multiple(&mut rng, 5)
                .into_iter()
                .map(ToString::to_string)
                .collect();

            cx.answer(format!("You have rolled: {}", random_emojis.join("")))
                .await?
        }
        Command::UsernameAndAge { username, age } => {
            cx.answer(format!(
                "Your username is @{} and age is {}.",
                username, age
            ))
            .await?
        }
    };

    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    teloxide::enable_logging!();
    log::info!("Starting dices_bot...");

    let bot = Bot::from_env().auto_send();

    let bot_name = "Emoji Album Bot";
    teloxide::commands_repl(bot, bot_name, answer).await;
}
