use crate::storage::Storage;
use crate::types::{EmojiRow, Quantity, User, Username};
use indexmap::IndexMap;
use std::collections::HashMap;

pub struct InMemoryStorage {
    users_emojis: HashMap<Username, IndexMap<EmojiRow, Quantity>>,
}

impl InMemoryStorage {
    pub fn new() -> Self {
        Self {
            users_emojis: HashMap::new(),
        }
    }
}

impl Storage for InMemoryStorage {
    fn get_user(&self, username: &Username) -> User {}
}
