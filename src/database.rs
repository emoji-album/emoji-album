use crate::storage::Storage;
use crate::types::{Command, User, Username};

pub fn fetch(
    storage: &dyn Storage,
    command: &Command,
    username: &Username,
) -> Result<DatabaseState, DatabaseError> {
    let state = match command {
        Command::Start => DatabaseState::Start {
            user: fetch_user(storage, username)?,
        },
        Command::Roll => DatabaseState::Roll,
        Command::Album => DatabaseState::Album,
        Command::Send(_, _, _) => DatabaseState::Send,
    };

    Ok(state)
}

fn fetch_user(storage: &dyn Storage, username: &Username) -> Result<User, DatabaseError> {
    let user = storage.get_user(username);

    Ok(user)
}

pub enum DatabaseState {
    Start { user: User },
    Roll,
    Album,
    Send,
}

#[derive(Debug)]
pub struct DatabaseError;
