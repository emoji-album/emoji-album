use std::cmp::Ordering;

pub type Username = String;
pub type Emoji = String;
pub type Quantity = usize;
pub type ReplyMsg = String;

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
