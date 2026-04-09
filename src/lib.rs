//! Doint - Discord economy bot

pub mod discord;
pub mod knob;
pub mod schema;
pub mod stock;

// Re-exports
pub use discord::prelude::*;
pub use knob::*;
pub use schema::*;
pub use stock::*;

/// Application state
#[derive(Debug, Clone)]
pub struct AppState {
    /// Stock market manager
    pub stock_manager: StockManager,
}

impl AppState {
    /// Create new application state
    pub fn new() -> Self {
        Self {
            stock_manager: StockManager::new(StockConfig::default()),
        }
    }
}