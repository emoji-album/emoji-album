use dotenv::dotenv;
use std::env;

// const EMOJIS: [char; 3] = ['😘', '🤗', '🦀'];

fn main() {
    dotenv().ok();

    println!("env: {}", env::var("TELOXIDE_TOKEN").unwrap());
}
