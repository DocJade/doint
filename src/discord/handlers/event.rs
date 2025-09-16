// Handles / dispatches discord related events
use poise::serenity_prelude as serenity;
use crate::types::serenity_types::{Context, Error, Data};

pub async fn handle_discord_event(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Ready { data_about_bot, .. } => {
            println!("Ready event. Logged in as {}", data_about_bot.user.name);
        }
        _ => {}
    }
    Ok(())
}