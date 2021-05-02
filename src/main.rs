use dotenv::dotenv;
use emoji_album::handlers::handle_update;
use emoji_album::storage::in_memory::InMemoryStorage;
use futures::StreamExt;
use std::env;
use telegram_bot::Api;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN not set");

    let api = Api::new(token);

    let mut stream = api.stream();

    let in_memory_storage = InMemoryStorage::new();

    while let Some(update) = stream.next().await {
        let _ = handle_update(update, &api, &in_memory_storage).await;
    }
}
