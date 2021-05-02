use crate::types::{Command, Username};
use crate::storage::Storage;

pub fn fetch(storage: &dyn Storage, command: &Command) -> Result<DatabaseState, DatabaseError> {
    let state = match command {
        Command::Start => DatabaseState::Start{
            user: fetch_user(storage, username)?,
        },
        Command::Roll => DatabaseState::Roll,
        Command::Album => DatabaseState::Album,
        Command::Send(_, _, _) => DatabaseState::Send,
    };

    Ok(state)
}

fn fetch_user(storage: &dyn Storage) -> Result<User, DatabaseError> {
    let user = storage.
}

pub enum DatabaseState {
    Start{ user: User },
    Roll,
    Album,
    Send,
}

pub struct User {
    username: Username,
}

#[derive(Debug)]
pub struct DatabaseError;
