use telegram_bot::{Api, CanSendMessage, Error, Message};

pub async fn send_message(api: &Api, message: &Message, text: String) -> Result<(), Error> {
    api.send(message.chat.text(text)).await?;
    Ok(())
}
