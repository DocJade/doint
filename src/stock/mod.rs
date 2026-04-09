//! Stock market simulation for Doint economy

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use rand::Rng;
use serde::{Deserialize, Serialize};
use tokio::time;

/// Stock market configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockConfig {
    /// Base price of Doint stock
    pub base_price: f64,
    /// Volatility factor (higher = more volatile)
    pub volatility: f64,
    /// Message activity window size (in minutes)
    pub message_window_minutes: usize,
    /// Velocity of money calculation window (in transactions)
    pub velocity_window: usize,
    /// Update interval in seconds
    pub update_interval_secs: u64,
}

impl Default for StockConfig {
    fn default() -> Self {
        Self {
            base_price: 100.0,
            volatility: 0.15,
            message_window_minutes: 60 * 24, // 24 hours
            velocity_window: 1000,
            update_interval_secs: 60, // Update every minute
        }
    }
}

/// Stock market data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockData {
    pub timestamp: u64,
    pub price: f64,
    pub change: f64,
    pub velocity_of_money: f64,
    pub message_activity: f64,
    pub gini_coefficient: f64,
    pub bank_reserves_ratio: f64,
}

/// Stock market simulator
#[derive(Debug)]
pub struct StockMarket {
    config: StockConfig,
    current_price: f64,
    message_history: VecDeque<u64>, // Timestamps of messages
    transaction_history: VecDeque<(u64, f64)>, // (timestamp, amount)
    price_history: VecDeque<StockData>,
    last_update: SystemTime,
}

impl StockMarket {
    /// Create a new stock market
    pub fn new(config: StockConfig) -> Self {
        Self {
            config,
            current_price: config.base_price,
            message_history: VecDeque::with_capacity(10000),
            transaction_history: VecDeque::with_capacity(10000),
            price_history: VecDeque::with_capacity(1000),
            last_update: SystemTime::now(),
        }
    }

    /// Record a message for activity tracking
    pub fn record_message(&mut self) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.message_history.push_back(timestamp);
        
        // Keep only recent messages within our window
        let cutoff = timestamp.saturating_sub((self.config.message_window_minutes * 60) as u64);
        while self.message_history.front().map_or(false, |&t| t < cutoff) {
            self.message_history.pop_front();
        }
    }

    /// Record a transaction for velocity of money calculation
    pub fn record_transaction(&mut self, amount: f64) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.transaction_history.push_back((timestamp, amount));
        
        // Keep only recent transactions
        while self.transaction_history.len() > self.config.velocity_window {
            self.transaction_history.pop_front();
        }
    }

    /// Calculate velocity of money
    fn calculate_velocity_of_money(&self) -> f64 {
        if self.transaction_history.len() < 10 {
            return 1.0; // Default value
        }
        
        let total_amount: f64 = self.transaction_history.iter().map(|(_, amount)| amount).sum();
        let avg_amount = total_amount / self.transaction_history.len() as f64;
        
        // Normalize to a reasonable range
        (avg_amount / 100.0).clamp(0.1, 10.0)
    }

    /// Calculate message activity level
    fn calculate_message_activity(&self) -> f64 {
        if self.message_history.is_empty() {
            return 0.5; // Neutral value
        }
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        // Count messages in last hour
        let hour_ago = now.saturating_sub(3600);
        let recent_count = self.message_history.iter().filter(|&&t| t >= hour_ago).count();
        
        // Normalize: 0-100 messages per hour = 0.0-1.0
        (recent_count as f64 / 100.0).clamp(0.0, 1.0)
    }

    /// Calculate Gini coefficient (wealth disparity)
    /// This would normally come from actual user wealth data
    /// For now, we simulate with some randomness
    fn calculate_gini_coefficient(&self) -> f64 {
        let mut rng = rand::thread_rng();
        
        // Base Gini with some randomness
        let base_gini = 0.3; // Reasonable base inequality
        let noise = rng.gen_range(-0.05..0.05);
        
        (base_gini + noise).clamp(0.0, 1.0)
    }

    /// Calculate bank reserves ratio
    /// This would normally come from actual bank data
    fn calculate_bank_reserves_ratio(&self) -> f64 {
        let mut rng = rand::thread_rng();
        
        // Simulate bank reserves between 0.1 and 0.9
        let base_reserves = 0.5;
        let noise = rng.gen_range(-0.1..0.1);
        
        (base_reserves + noise).clamp(0.1, 0.9)
    }

    /// Update stock price based on all factors
    pub fn update_price(&mut self) -> StockData {
        let velocity = self.calculate_velocity_of_money();
        let activity = self.calculate_message_activity();
        let gini = self.calculate_gini_coefficient();
        let reserves = self.calculate_bank_reserves_ratio();
        
        let mut rng = rand::thread_rng();
        
        // Calculate factors
        let velocity_factor = (velocity - 1.0) * 0.1; // ±10% effect
        let activity_factor = (activity - 0.5) * 0.2; // ±10% effect
        let gini_factor = (0.5 - gini) * 0.1; // Lower Gini (more equality) is better
        let reserves_factor = (reserves - 0.5) * 0.1; // Higher reserves is better
        
        // Random noise for volatility
        let noise = rng.gen_range(-self.config.volatility..self.config.volatility);
        
        // Combine all factors
        let total_change = velocity_factor + activity_factor + gini_factor + reserves_factor + noise;
        
        // Apply change (limit to ±20% per update)
        let change = total_change.clamp(-0.2, 0.2);
        let old_price = self.current_price;
        self.current_price *= 1.0 + change;
        
        // Ensure price doesn't go too low
        self.current_price = self.current_price.max(0.01);
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        let data = StockData {
            timestamp,
            price: self.current_price,
            change,
            velocity_of_money: velocity,
            message_activity: activity,
            gini_coefficient: gini,
            bank_reserves_ratio: reserves,
        };
        
        self.price_history.push_back(data.clone());
        if self.price_history.len() > 1000 {
            self.price_history.pop_front();
        }
        
        self.last_update = SystemTime::now();
        data
    }

    /// Get current stock data
    pub fn get_current_data(&self) -> StockData {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        StockData {
            timestamp,
            price: self.current_price,
            change: 0.0,
            velocity_of_money: self.calculate_velocity_of_money(),
            message_activity: self.calculate_message_activity(),
            gini_coefficient: self.calculate_gini_coefficient(),
            bank_reserves_ratio: self.calculate_bank_reserves_ratio(),
        }
    }

    /// Get price history
    pub fn get_price_history(&self, limit: usize) -> Vec<StockData> {
        self.price_history.iter().rev().take(limit).cloned().collect()
    }
}

/// Stock market manager
#[derive(Debug, Clone)]
pub struct StockManager {
    market: Arc<Mutex<StockMarket>>,
}

impl StockManager {
    /// Create a new stock manager
    pub fn new(config: StockConfig) -> Self {
        Self {
            market: Arc::new(Mutex::new(StockMarket::new(config))),
        }
    }

    /// Start the stock market update loop
    pub async fn start_update_loop(&self) {
        let market = Arc::clone(&self.market);
        let config = {
            let market = market.lock().unwrap();
            market.config.clone()
        };
        
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(config.update_interval_secs));
            
            loop {
                interval.tick().await;
                
                let mut market = market.lock().unwrap();
                market.update_price();
            }
        });
    }

    /// Record a message
    pub fn record_message(&self) {
        let mut market = self.market.lock().unwrap();
        market.record_message();
    }

    /// Record a transaction
    pub fn record_transaction(&self, amount: f64) {
        let mut market = self.market.lock().unwrap();
        market.record_transaction(amount);
    }

    /// Get current stock data
    pub fn get_current_data(&self) -> StockData {
        let market = self.market.lock().unwrap();
        market.get_current_data()
    }

    /// Get price history
    pub fn get_price_history(&self, limit: usize) -> Vec<StockData> {
        let market = self.market.lock().unwrap();
        market.get_price_history(limit)
    }
}
