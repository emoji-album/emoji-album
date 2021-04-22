use dotenv::dotenv;
use rand::rngs::StdRng;
use rand::seq::IteratorRandom;
use rand::FromEntropy;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::string::ToString;
use std::sync::{Arc, Mutex};

use futures::StreamExt;
use telegram_bot::{Api, CanSendMessage, Error, MessageKind, UpdateKind};

type Username = String;

lazy_static::lazy_static! {
    static ref USER_EMOJIS: Arc<Mutex<HashMap<Username, Vec<char>>>> = Arc::new(Mutex::new(HashMap::new()));
    static ref EMOJI_FILE: String = fs::read_to_string("emojis.csv").unwrap();
    static ref EMOJIS: Vec<&'static str> = EMOJI_FILE.trim().split('\n').collect();
}

fn roll() -> String {
    let mut rng = StdRng::from_entropy();

    let random_emojis: Vec<String> = EMOJIS
        .iter()
        .choose_multiple(&mut rng, 5)
        .into_iter()
        .map(ToString::to_string)
        .collect();

    random_emojis.join("")
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();

    let token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN not set");

    let api = Api::new(token);

    let mut stream = api.stream();

    while let Some(update) = stream.next().await {
        let update = update?;

        if let UpdateKind::Message(message) = update.kind {
            if let MessageKind::Text { ref data, .. } = message.kind {
                println!("<{}>: {}", &message.from.id, data);

                match &data[..] {
                    "/roll" => {
                        api.send(message.chat.text(format!("You have rolled: {}", roll())))
                            .await?;
                    }
                    _ => println!("no match"),
                };
            }
        }
    }

    Ok(())
}
