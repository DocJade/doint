use rand::Rng;
use std::time::{Duration, Instant};

/// Market data structure for simulating real-time data streams.
#[derive(Debug, Clone)]
pub struct MarketData {
    pub symbol: String,
    pub price: f64,
    pub volume: u64,
    pub timestamp: Instant,
}

impl MarketData {
    pub fn new(symbol: &str) -> Self {
        let mut rng = rand::thread_rng();
        MarketData {
            symbol: symbol.to_string(),
            price: rng.gen_range(100.0..1000.0),
            volume: rng.gen_range(100..10000),
            timestamp: Instant::now(),
        }
    }

    pub fn update(&mut self) {
        let mut rng = rand::thread_rng();
        self.price = rng.gen_range(100.0..1000.0);
        self.volume = rng.gen_range(100..10000);
        self.timestamp = Instant::now();
    }

    pub fn simulate_data_stream(&self) -> Vec<MarketData> {
        let mut data_stream = Vec::new();
        for _ in 0..10 {
            let mut data = self.clone();
            data.update();
            data_stream.push(data);
            std::thread::sleep(Duration::from_secs(1));
        }
        data_stream
    }
}
