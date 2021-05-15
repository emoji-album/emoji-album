use crate::types::{Command, Emoji, EmojiRow, Quantity, ReplyMsg, Username};
use indexmap::{map::Entry as IndexMapEntry, IndexMap};
use itertools::Itertools;
use rand::rngs::StdRng;
use rand::seq::IteratorRandom;
use rand::FromEntropy;
use std::collections::hash_map::Entry as HashMapEntry;
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex};

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

impl Command {
    pub fn execute(self, msg_username: Username) -> Result<ReplyMsg, String> {
        match self {
            Command::Start => self.start(),
            Command::Roll => self.roll(msg_username),
            Command::Album => self.album(msg_username),
            Command::Send(ref emoji, quantity, ref username) => {
                self.send(emoji, quantity, &msg_username, username)
            }
        }
    }

    fn start(&self) -> Result<ReplyMsg, String> {
        Ok("Welcome to emoji album!\n\n🎲 Send /roll to get your first emojis!\n\n📖 Send /album to see all your emojis!".into())
    }

    fn roll(&self, username: Username) -> Result<ReplyMsg, String> {
        let rolled_emojis = generate_random_emojis();

        add_emojis_to_album(username, &rolled_emojis);

        Ok(format!(
            "You have rolled:\n\n\n{}",
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
        )
        .into())
    }

    fn album(&self, username: Username) -> Result<ReplyMsg, String> {
        let lock = USERS_EMOJIS.lock().unwrap();

        match lock.get(&username) {
            Some(emojis_map) => {
                let emoji_album = render_emoji_album(emojis_map);

                Ok(format!("Your album:\n\n\n{}", emoji_album).into())
            }
            None => Ok("You still have no emojis in your album! Type /roll to get some!".into()),
        }
    }

    fn send(
        &self,
        emoji: &Emoji,
        quantity: Quantity,
        from: &Username,
        to: &Username,
    ) -> Result<ReplyMsg, String> {
        let mut lock = USERS_EMOJIS.lock().unwrap();

        let user_from = lock.entry(from.into()).or_insert(IndexMap::new());

        // TODO:
        // 1. only remove quantity from origin if the target user exists

        let related_emoji_row = EMOJI_TABLE
            .iter()
            .find(|EmojiRow { icon, .. }| icon == emoji)
            .ok_or_else(|| {
                "Emoji not valid, or there's a space missing between the emoji and the username"
            })?;

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
        )
        .into())
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
