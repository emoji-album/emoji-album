use crate::error::Error;
use crate::types::{User, Username};

pub mod in_memory;

pub trait Storage {
    fn has_entity(&self, identifier: &str) -> Result<(), Error>;
    fn insert(&mut self, username: &Username) -> Result<User, Error>;
}
