use crate::storage::Storage;

pub struct InMemoryStorage;

impl InMemoryStorage {
    pub fn new() -> Self {
        Self
    }
}

impl Storage for InMemoryStorage {

}
