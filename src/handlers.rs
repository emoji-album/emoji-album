use crate::telegram::send_message;
use crate::types::{Command, ReplyMsg};
use telegram_bot::{Api, Error, Message, MessageKind, Update, UpdateKind};

fn format_error(message: &str) -> String {
    format!("Error: {}", message)
}

fn handle_command(username: Option<&String>, text: &str) -> Result<ReplyMsg, String> {
    let command = Command::parse(text)?;

    let msg_username = username.ok_or_else(|| "You must register an username in your Telegram in order to use this bot. Set a username in your telegram app settings.")?.to_owned();

    command.execute(msg_username)
}

async fn handle_message(api: &Api, message: &Message) -> Result<(), Error> {
    if let MessageKind::Text { ref data, .. } = message.kind {
        let maybe_username = message.from.username.as_ref();

        println!("<{:?}>: {}", maybe_username, data);

        match handle_command(maybe_username, &data[..]) {
            Ok(success_reply_msg) => send_message(api, message, success_reply_msg),
            Err(error_reply_msg) => {
                send_message(api, message, format_error(&error_reply_msg).into())
            }
        }
        .await?;
    };

    Ok(())
}

pub async fn handle_update(update: Result<Update, Error>, api: &Api) -> Result<(), Error> {
    if let UpdateKind::Message(message) = update?.kind {
        handle_message(api, &message).await?;
    }

    Ok(())
}
