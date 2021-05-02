use crate::database;
use crate::telegram::send_message;
use crate::types::{Command, ReplyMsg};
use crate::storage::Storage;
use telegram_bot::{Api, Error, Message, MessageKind, Update, UpdateKind};

fn format_error(message: &str) -> String {
    format!("Error: {}", message)
}

fn handle_command(storage: &dyn Storage, username: Option<&String>, text: &str) -> Result<ReplyMsg, ReplyMsg> {
    let command = Command::parse(text)?;

    let msg_username = username.ok_or_else(|| "You must register an username in your Telegram in order to use this bot. Set a username in your telegram app settings.")?.to_owned();

    let database_state = database::fetch(storage, &command).map_err(|db_error| {
        println!("database error: {:?}", db_error);
        "Internal server error"
    })?;

    command.execute(database_state, msg_username)
}

async fn handle_message(api: &Api, storage: &dyn Storage, message: &Message) -> Result<(), Error> {
    if let MessageKind::Text { ref data, .. } = message.kind {
        let maybe_username = message.from.username.as_ref();

        println!("<{:?}>: {}", maybe_username, data);

        let reply_msg = match handle_command(storage, maybe_username, &data[..]) {
            Ok(success_reply_msg) => success_reply_msg,
            Err(error_reply_msg) => format_error(&error_reply_msg),
        };

        send_message(api, message, reply_msg).await?;
    };

    Ok(())
}

pub async fn handle_update(update: Result<Update, Error>, api: &Api, storage: &dyn Storage) -> Result<(), Error> {
    if let UpdateKind::Message(message) = update?.kind {
        handle_message(api, storage, &message).await?;
    }

    Ok(())
}
