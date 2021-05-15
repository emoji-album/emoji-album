use std::cmp::Ordering;

pub type Username = String;
pub type Emoji = String;
pub type Quantity = usize;

pub struct ReplyMsg {
    pub text: String,
    pub buttons: Vec<String>,
}

impl ReplyMsg {
    fn new(text: &str) -> Self {
        Self {
            text: text.to_owned(),
            buttons: vec![],
        }
    }
}

impl From<String> for ReplyMsg {
    fn from(text: String) -> Self {
        Self::new(&text)
    }
}

impl From<&str> for ReplyMsg {
    fn from(text: &str) -> Self {
        Self::new(text)
    }
}

pub enum Command {
    Start,
    Roll,
    Album,
    Send(Emoji, Quantity, Username),
}

#[derive(Hash, PartialEq, Eq, Clone)]
pub struct EmojiRow {
    pub icon: Emoji,
    pub collection: String,
    pub index: usize,
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
