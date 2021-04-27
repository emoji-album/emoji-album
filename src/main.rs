use dotenv::dotenv;
use indexmap::IndexMap;
use rand::rngs::StdRng;
use rand::seq::IteratorRandom;
use rand::FromEntropy;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::env;
use std::fs;
use std::string::ToString;
use std::sync::{Arc, Mutex};

use futures::StreamExt;
use telegram_bot::{Api, CanSendMessage, Error, Message, MessageKind, Update, UpdateKind};

type Username = String;
type Emoji = String;
type Quantity = usize;

lazy_static::lazy_static! {
    static ref USERS_EMOJIS: Arc<Mutex<HashMap<Username, IndexMap<Emoji, Quantity>>>> = Arc::new(Mutex::new(HashMap::new()));

    static ref EMOJI_FILE: String = fs::read_to_string("emojis.csv").unwrap();
    static ref EMOJIS: Vec<&'static str> = EMOJI_FILE.trim().split('\n').collect();
}

enum Command {
    Start,
    Roll,
    Album,
    Send(Emoji, Quantity, Username),
}

impl TryFrom<&str> for Command {
    type Error = &'static str;

    fn try_from(message: &str) -> Result<Self, Self::Error> {
        if message.starts_with("/start") {
            return Ok(Self::Start);
        }

        if message.starts_with("/roll") {
            return Ok(Self::Roll);
        }

        if message.starts_with("/album") {
            return Ok(Self::Album);
        }

        if message.starts_with("/send") {
            let params: Vec<&str> = message.split(' ').skip(1).collect();

            // TODO:
            // 1. needs error handling on the `@` of the string,
            // right now it doesn't check, it just removes;
            // 2. needs to remove hardcoded `1` in Quantity.
            return Ok(Self::Send(
                params[0].to_string(),
                1,
                params[1][1..].to_string(),
            ));
        }

        Err("no match")
    }
}

impl Command {
    async fn execute(self, api: &Api, message: &Message) -> Result<(), Error> {
        let msg_username = message.from.username.as_ref().unwrap().to_owned();

        match self {
            Command::Start => self.start(api, message).await,
            Command::Roll => self.roll(api, message, msg_username).await,
            Command::Album => self.album(api, message, msg_username).await,
            Command::Send(ref emoji, quantity, ref username) => {
                self.send(api, message, emoji, quantity, &msg_username, username)
                    .await
            }
        }?;

        Ok(())
    }

    async fn start(&self, api: &Api, message: &Message) -> Result<(), Error> {
        api.send(message.chat.text(
                "Welcome to emoji album!\n\n🎲 Send /roll to get your first emojis!\n\n📖 Send /album to see all your emojis!"
            ))
        .await?;

        Ok(())
    }

    async fn roll(&self, api: &Api, message: &Message, username: Username) -> Result<(), Error> {
        let rolled_emojis = generate_random_emojis();

        add_emojis_to_album(username, &rolled_emojis);

        api.send(message.chat.text(format!(
                "You have rolled:\n\n{}\n\nSend /album to see all your emojis!",
                rolled_emojis
                    .into_iter()
                    .rev()
                    .collect::<Vec<String>>()
                    .join(" ")
            )))
        .await?;

        Ok(())
    }

    async fn album(&self, api: &Api, message: &Message, username: Username) -> Result<(), Error> {
        let lock = USERS_EMOJIS.lock().unwrap();

        match lock.get(&username) {
            Some(emojis_map) => {
                let emoji_album = render_emoji_album(emojis_map);

                api.send(message.chat.text(format!(
                    "Your album:\n\n{}\n\nSend /roll to get more emojis",
                    emoji_album
                )))
                .await?;
            }
            None => {
                api.send(
                    message
                        .chat
                        .text("You still have no emojis in your album! Type /roll to get some!"),
                )
                .await?;
            }
        };

        Ok(())
    }

    async fn send(
        &self,
        api: &Api,
        message: &Message,
        emoji: &Emoji,
        quantity: Quantity,
        from: &Username,
        to: &Username,
    ) -> Result<(), Error> {
        let mut lock = USERS_EMOJIS.lock().unwrap();

        let user_from = lock.entry(from.into()).or_insert(IndexMap::new());

        // TODO:
        // 1. validate if there is some before giving to the other person
        // 2. also, if it gets to zero, remove key
        let quantity_from = user_from.entry(emoji.to_owned()).or_insert(1);

        (*quantity_from) -= 1;

        drop(lock);

        let mut lock = USERS_EMOJIS.lock().unwrap();

        let user_to = lock.entry(to.into()).or_insert(IndexMap::new());
        let quantity_to = user_to.entry(emoji.to_owned()).or_insert(0);

        (*quantity_to) += 1;

        api.send(message.chat.text(format!(
            "You have succefuly sent {} {} to @{}!",
            quantity, emoji, to
        )))
        .await?;

        Ok(())
    }
}

fn generate_random_emojis() -> Vec<Emoji> {
    let mut rng = StdRng::from_entropy();

    let random_emojis: Vec<String> = EMOJIS
        .iter()
        .choose_multiple(&mut rng, 5)
        .into_iter()
        .map(ToString::to_string)
        .collect();

    random_emojis
}

fn add_emojis_to_album(album: Username, emojis: &Vec<Emoji>) {
    let mut lock = USERS_EMOJIS.lock().unwrap();
    let user_emojis = lock.entry(album).or_insert(IndexMap::new());

    for emoji in emojis {
        let quantity = user_emojis.entry(emoji.to_owned()).or_insert(0);
        (*quantity) += 1;
    }
}

fn render_emoji_album(emojis_map: &IndexMap<Emoji, Quantity>) -> String {
    emojis_map
        .iter()
        .rev()
        .map(|(emoji, quantity)| {
            std::iter::repeat(emoji.to_owned())
                .take(*quantity)
                .collect::<String>()
        })
        .map(|mut same_emoji_line| {
            same_emoji_line.push_str(" ");
            same_emoji_line
        })
        .collect()
}

async fn handle_message(api: &Api, message: &Message) -> Result<(), Error> {
    if let MessageKind::Text { ref data, .. } = message.kind {
        println!("<{:?}>: {}", &message.from.username, data);

        match Command::try_from(&data[..]) {
            Ok(command) => command.execute(&api, message).await?,
            Err(error_msg) => println!("{}", error_msg),
        }
    };

    Ok(())
}

async fn handle_update(update: Result<Update, Error>, api: &Api) -> Result<(), Error> {
    if let UpdateKind::Message(message) = update?.kind {
        handle_message(api, &message).await?;
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();

    let token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN not set");

    let api = Api::new(token);

    let mut stream = api.stream();

    while let Some(update) = stream.next().await {
        let _ = handle_update(update, &api).await;
    }

    Ok(())
}
