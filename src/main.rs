use dotenv::dotenv;
use indexmap::{map::Entry as IndexMapEntry, IndexMap};
use itertools::Itertools;
use rand::rngs::StdRng;
use rand::seq::IteratorRandom;
use rand::FromEntropy;
use std::cmp::Ordering;
use std::collections::hash_map::Entry as HashMapEntry;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::env;
use std::fs;
use std::string::ToString;
use std::sync::{Arc, Mutex};

use futures::StreamExt;
use telegram_bot::{Api, CanSendMessage, Error, Message, MessageKind, Update, UpdateKind};

#[derive(Hash, PartialEq, Eq, Clone)]
struct EmojiRow {
    icon: Emoji,
    collection: String,
    index: usize,
}

impl Ord for EmojiRow {
    fn cmp(&self, other: &Self) -> Ordering {
        self.index.cmp(&other.index)
    }
}

impl PartialOrd for EmojiRow {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

type Username = String;
type Emoji = String;
type Quantity = usize;
type ReplyMsg = String;

lazy_static::lazy_static! {
    static ref USERS_EMOJIS: Arc<Mutex<HashMap<Username, IndexMap<EmojiRow, Quantity>>>> = Arc::new(Mutex::new(HashMap::new()));

    static ref EMOJI_FILE: String = fs::read_to_string("emojis.csv").unwrap();
    static ref EMOJI_TABLE: Vec<EmojiRow> = EMOJI_FILE.trim().split('\n').enumerate().map(|(i, text_row)| {
        let row_vec: Vec<&str> = text_row.split(',').collect();

        EmojiRow {
            icon: row_vec[0].to_owned(),
            collection: row_vec[1].to_owned(),
            index: i,
        }
    }).collect();
}

fn parse_username(text: &str) -> Result<String, &'static str> {
    if text.is_empty() {
        return Err("/send username cannot be empty. To send emojis to someone follow the format: `/send QUANTITY EMOJI @USERNAME`");
    }

    if text.chars().nth(0).unwrap() == '@' {
        return Ok(text[1..].to_owned());
    }

    Ok(text.to_owned())
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
            if message != "/start" {
                return Err("/start command accepts no arguments");
            }

            return Ok(Self::Start);
        }

        if message.starts_with("/roll") {
            if message != "/roll" {
                return Err("/roll command accepts no arguments");
            }

            return Ok(Self::Roll);
        }

        if message.starts_with("/album") {
            if message != "/album" {
                return Err("/album command accepts no arguments");
            }

            return Ok(Self::Album);
        }

        if message.starts_with("/send") {
            let params: Vec<&str> = message.split(' ').skip(1).collect();

            if params.len() < 2 {
                return Err("To send emojis to someone follow the format `/send QUANTITY EMOJI @USERNAME` like `/send 3 😍 @coolusername`. The quantity is optional.");
            }

            let emoji;
            let quantity = if params.len() == 2 {
                emoji = params[0].to_string();
                1
            } else {
                emoji = params[1].to_string();
                params[0].parse().map_err(|_| {
                    "The quantity parameter should be a integer number, for example: 3"
                })?
            };
            let username = parse_username(params.last().unwrap())?;

            return Ok(Self::Send(emoji, quantity, username));
        }

        Err("Command not found. Type or press / to see all available commands.")
    }
}

impl Command {
    fn execute(self, msg_username: Username) -> Result<ReplyMsg, ReplyMsg> {
        match self {
            Command::Start => self.start(),
            Command::Roll => self.roll(msg_username),
            Command::Album => self.album(msg_username),
            Command::Send(ref emoji, quantity, ref username) => {
                self.send(emoji, quantity, &msg_username, username)
            }
        }
    }

    fn start(&self) -> Result<ReplyMsg, ReplyMsg> {
        Ok("Welcome to emoji album!\n\n🎲 Send /roll to get your first emojis!\n\n📖 Send /album to see all your emojis!".to_string())
    }

    fn roll(&self, username: Username) -> Result<ReplyMsg, ReplyMsg> {
        let rolled_emojis = generate_random_emojis();

        add_emojis_to_album(username, &rolled_emojis);

        Ok(format!(
            "You have rolled:\n\n\n{}\n\nSend /album to see all your emojis!",
            rolled_emojis
                .into_iter()
                .group_by(|emoji_row| emoji_row.collection.clone())
                .into_iter()
                .map(|(collection, group)| {
                    format!(
                        "{} Collection\n{}\n\n",
                        collection,
                        group.map(|emoji| emoji.icon.clone()).join(" ")
                    )
                })
                .collect::<Vec<String>>()
                .join(" ")
        ))
    }

    fn album(&self, username: Username) -> Result<ReplyMsg, ReplyMsg> {
        let lock = USERS_EMOJIS.lock().unwrap();

        match lock.get(&username) {
            Some(emojis_map) => {
                let emoji_album = render_emoji_album(emojis_map);

                Ok(format!(
                    "Your album:\n\n\n{}\n\nSend /roll to get more emojis",
                    emoji_album
                ))
            }
            None => {
                Ok("You still have no emojis in your album! Type /roll to get some!".to_string())
            }
        }
    }

    fn send(
        &self,
        emoji: &Emoji,
        quantity: Quantity,
        from: &Username,
        to: &Username,
    ) -> Result<ReplyMsg, ReplyMsg> {
        let mut lock = USERS_EMOJIS.lock().unwrap();

        let user_from = lock.entry(from.into()).or_insert(IndexMap::new());

        // TODO:
        // 1. only remove quantity from origin if the target user exists

        let related_emoji_row = EMOJI_TABLE
            .iter()
            .find(|EmojiRow { icon, .. }| icon == emoji)
            .unwrap();

        let emoji_row = EmojiRow {
            icon: emoji.to_owned(),
            collection: related_emoji_row.collection.clone(),
            index: related_emoji_row.index,
        };
        let mut quantity_from = match user_from.entry(emoji_row) {
            IndexMapEntry::Vacant(_) => {
                return Err(format!("You don't have any {} to send", emoji))
            }
            IndexMapEntry::Occupied(ref entry) if (*entry.get()) < quantity => {
                return Err(format!("You don't have enough {} to send", emoji))
            }
            IndexMapEntry::Occupied(quantity_from) => quantity_from,
        };

        (*quantity_from.get_mut()) -= quantity;

        if *quantity_from.get() == 0 {
            quantity_from.remove();
        }

        let mut user_to = match lock.entry(to.into()) {
            HashMapEntry::Vacant(_) => return Err(format!("Could not find user @{}. Make sure the user has contacted @EmojiAlbumBot at least once", to)),
            HashMapEntry::Occupied(user_to) => {
                user_to
            },
        };

        let related_emoji_row = EMOJI_TABLE
            .iter()
            .find(|EmojiRow { icon, .. }| icon == emoji)
            .unwrap();

        let emoji_row = EmojiRow {
            icon: emoji.to_owned(),
            collection: related_emoji_row.collection.clone(),
            index: related_emoji_row.index,
        };
        user_to.get_mut().entry(emoji_row).or_insert(quantity);

        Ok(format!(
            "You have successfully sent {} {} to @{}!",
            quantity, emoji, to
        ))
    }
}

fn generate_random_emojis() -> Vec<EmojiRow> {
    let mut rng = StdRng::from_entropy();

    let random_emojis: Vec<EmojiRow> = EMOJI_TABLE
        .iter()
        .choose_multiple(&mut rng, 5)
        .into_iter()
        .map(Clone::clone)
        .collect();

    random_emojis
}

fn add_emojis_to_album(album: Username, emoji_rows: &Vec<EmojiRow>) {
    let mut lock = USERS_EMOJIS.lock().unwrap();
    let user_emojis = lock.entry(album).or_insert(IndexMap::new());

    for emoji_row in emoji_rows {
        let quantity = user_emojis.entry(emoji_row.clone()).or_insert(0);
        (*quantity) += 1;
    }
}

fn render_emoji_album(emojis_map: &IndexMap<EmojiRow, Quantity>) -> String {
    let mut emojis_map = emojis_map.clone();
    emojis_map.sort_by(|k1, _v1, k2, _v2| k1.cmp(k2));

    emojis_map
        .iter()
        .group_by(|(emoji_row, _)| emoji_row.collection.clone())
        .into_iter()
        .map(|(collection, group)| {
            format!(
                "{} Collection\n{}\n\n",
                collection,
                group
                    .map(|(emoji, quantity)| std::iter::repeat(emoji.icon.clone())
                        .take(*quantity)
                        .collect::<String>())
                    .join(" ")
            )
        })
        // TODO: separate repeated emojis later
        .collect()
}

async fn send_message(api: &Api, message: &Message, text: String) -> Result<(), Error> {
    api.send(message.chat.text(text)).await?;
    Ok(())
}

fn format_error(message: &str) -> String {
    format!("Error: {}", message)
}

fn handle_command(username: Option<&String>, text: &str) -> ReplyMsg {
    match Command::try_from(text) {
        Ok(command) => {
            let msg_username = match username {
                Some(msg_username) => msg_username.to_owned(),
                None => return format_error("You must register an username in your Telegram in order to use this bot. Set a username in your telegram app settings."),
            };

            match command.execute(msg_username) {
                Ok(reply_msg) => reply_msg,
                Err(reply_msg) => format_error(&reply_msg),
            }
        }
        Err(error_msg) => format_error(error_msg),
    }
}

async fn handle_message(api: &Api, message: &Message) -> Result<(), Error> {
    if let MessageKind::Text { ref data, .. } = message.kind {
        let maybe_username = message.from.username.as_ref();

        println!("<{:?}>: {}", maybe_username, data);

        let reply_msg = handle_command(maybe_username, &data[..]);

        send_message(api, message, reply_msg).await?;
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
