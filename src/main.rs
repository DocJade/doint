use doint::discord::start::create_client;
use dotenvy::dotenv;
use std::env;

#[tokio::main]
async fn main() {
    // Get env vars
    dotenv().ok();

    // Start logger.
    // Everything besides the bot is turned off, otherwise the logs are noisy.
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Off)
        .filter_module("doint", log::LevelFilter::Debug)
        .init();

    // load in our token
    let discord_token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN must be set");

    // And the database url
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Run the bot.
    let result = create_client(discord_token, database_url)
        .await
        .start()
        .await;
    if let Err(error) = result {
        panic!("Bot died! {error:#?}");
    }
}
