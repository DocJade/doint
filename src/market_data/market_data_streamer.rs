use crate::market_data::MarketData;
use std::thread;
use std::sync::{Arc, Mutex};

/// Stream data and send to bot system for processing.
pub fn start_market_data_stream(symbol: &str) {
    let data = Arc::new(Mutex::new(MarketData::new(symbol)));
    thread::spawn(move || {
        loop {
            let mut data = data.lock().unwrap();
            let data_stream = data.simulate_data_stream();
            for market_data in data_stream {
                // In a real system, here we would send data to the bot for processing.
                println!("Received market data: {:?}", market_data);
            }
        }
    });
}
