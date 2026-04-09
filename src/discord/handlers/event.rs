//! Event handlers for Discord

use crate::AppState;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        // Record message for stock market activity
        if !msg.author.bot && !msg.content.is_empty() {
            let data = ctx.data.read().await;
            if let Some(state) = data.get::<AppState>() {
                state.stock_manager.record_message();
            }
        }
        
        // Handle commands (existing code would go here)
        // ...
    }

    async fn ready(&self, _ctx: Context, ready: Ready) {
        log::info!("{} is connected!", ready.user.name);
    }
}