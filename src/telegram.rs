use crate::types::ReplyMsg;
use telegram_bot::types::*;
use telegram_bot::{Api, CanSendMessage, Error, Message};

pub async fn send_message(api: &Api, message: &Message, reply_msg: ReplyMsg) -> Result<(), Error> {
    let mut keyboard_markup = ReplyKeyboardMarkup::new();

    let inline_keyboard_row = vec![
        KeyboardButton::new("🎲 /roll"),
        KeyboardButton::new("📖 /album"),
    ];

    keyboard_markup.add_row(inline_keyboard_row);

    let mut send_msg = message.chat.text(reply_msg.text);

    let reply_markup: ReplyMarkup = keyboard_markup.into();

    send_msg.reply_markup(reply_markup);

    api.send(send_msg).await?;

    Ok(())
}
