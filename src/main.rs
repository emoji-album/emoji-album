use dotenv::dotenv;
use emoji_album::handle_update;
use futures::StreamExt;
use std::env;
use telegram_bot::Api;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN not set");

    let api = Api::new(token);

    let mut stream = api.stream();

    while let Some(update) = stream.next().await {
        let _ = handle_update(update, &api).await;
    }
}
