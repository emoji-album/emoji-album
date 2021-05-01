use crate::types::{Command, ReplyMsg};

pub fn fetch(command: &Command) -> Result<DatabaseState, ReplyMsg> {
    let state = match command {
        Command::Start => DatabaseState::Start,
        Command::Roll => DatabaseState::Roll,
        Command::Album => DatabaseState::Album,
        Command::Send(_, _, _) => DatabaseState::Send,
    };

    Ok(state)
}

pub enum DatabaseState {
    Start,
    Roll,
    Album,
    Send,
}
