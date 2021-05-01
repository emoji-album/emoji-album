use crate::types::Command;
use std::convert::TryFrom;

impl Command {
    pub fn parse(message: &str) -> Result<Self, &'static str> {
        Self::try_from(message)
    }

    fn parse_start(message: &str) -> Result<Self, &'static str> {
        if message != "/start" {
            return Err("/start command accepts no arguments");
        }

        return Ok(Self::Start);
    }

    fn parse_roll(message: &str) -> Result<Self, &'static str> {
        if message != "/roll" {
            return Err("/roll command accepts no arguments");
        }

        return Ok(Self::Roll);
    }

    fn parse_album(message: &str) -> Result<Self, &'static str> {
        if message != "/album" {
            return Err("/album command accepts no arguments");
        }

        return Ok(Self::Album);
    }

    fn parse_send(message: &str) -> Result<Self, &'static str> {
        let params: Vec<&str> = message.split(' ').skip(1).collect();

        if params.len() < 2 {
            return Err("To send emojis to someone follow the format `/send QUANTITY EMOJI @USERNAME` like `/send 3 😍 @coolusername`. The quantity is optional.");
        }

        let emoji;
        let quantity = if params.len() == 2 {
            emoji = params[0].to_string();
            1
        } else {
            emoji = params[1].to_string();
            params[0]
                .parse()
                .map_err(|_| "The quantity parameter should be a integer number, for example: 3")?
        };
        let username = parse_username(params.last().unwrap())?;

        return Ok(Self::Send(emoji, quantity, username));
    }
}

fn parse_username(text: &str) -> Result<String, &'static str> {
    if text.is_empty() {
        return Err("/send username cannot be empty. To send emojis to someone follow the format: `/send QUANTITY EMOJI @USERNAME`");
    }

    if text.chars().nth(0).unwrap() == '@' {
        return Ok(text[1..].to_owned());
    }

    Ok(text.to_owned())
}

impl TryFrom<&str> for Command {
    type Error = &'static str;

    fn try_from(message: &str) -> Result<Self, Self::Error> {
        if message.starts_with("/start") {
            return Self::parse_start(message);
        }

        if message.starts_with("/roll") {
            return Self::parse_roll(message);
        }

        if message.starts_with("/album") {
            return Self::parse_album(message);
        }

        if message.starts_with("/send") {
            return Self::parse_send(message);
        }

        Err("Command not found. Type or press / to see all available commands.")
    }
}
