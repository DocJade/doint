//! Main entry point for Doint bot

use doint::discord::start::start_bot;
use doint::AppState;
use std::error::Error;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging
    env_logger::init();
    
    // Create application state
    let state = AppState::new();
    
    // Start stock market updates
    state.stock_manager.start_update_loop().await;
    
    // Start Discord bot
    start_bot(state).await?;
    
    Ok(())
}