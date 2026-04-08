pub mod types;
pub mod data;
pub mod engine;
pub mod models;

use chrono::{Duration, Utc};
use types::types::{Candle, ReversePeriodConfig, LambdaWeights};
use engine::core::ReversePeriodEngine;

fn main() {
    // Setup Configuration
    let config = ReversePeriodConfig {
        lookback_20: 20,
        lookback_40: 40,
        lookback_60: 60,
        atr_period: 14,
        killzone_london_start: 8,
        killzone_london_end: 11,
        killzone_ny_start: 13,
        killzone_ny_end: 16,
        lambda_weights: LambdaWeights::default(),
    };

    let mut engine = ReversePeriodEngine::new(config);
    let mut candles = generate_mock_data(100); // Start with 100 bars

    // Simulate a market scenario
    for i in 0..50 {
        let new_candle = create_next_candle(&candles, i);
        candles.push(new_candle.clone());

        // Update Engine
        // Mock inputs: Volatility and Confidence
        let vol = 0.0005 + (i as f64 * 0.0001); 
        let conf = if i > 30 { 0.8 } else { 0.4 };

        engine.update(&candles, vol, conf);
        
        let state = engine.system_state();
        let r_score = {
            let matrix = &engine.engine.detectors.last_matrix;
            engine.engine.calculate_severity_score(matrix)
        };

        println!("Bar {}: State={:?}, R-Score={:.2}, Damped={}", i + 101, state, r_score, engine.is_damped());
    }
}

// --- Mock Data Generators ---

fn generate_mock_data(count: usize) -> Vec<Candle> {
    let mut data = Vec::new();
    let now = Utc::now();
    let mut price = 1.1000;

    for i in 0..count {
        let noise = (rand::random::<f64>() - 0.5) * 0.0010;
        price += noise;
        
        data.push(Candle {
            timestamp: now + Duration::hours(i as i64),
            open: price,
            high: price + 0.0005,
            low: price - 0.0005,
            close: price + noise * 0.5,
            volume: 1000.0,
        });
    }
    data
}

fn create_next_candle(history: &[Candle], step: usize) -> Candle {
    let last = history.last().unwrap();
    let mut price = last.close;
    
    // Simulate a trend then a reversal (exhaustion)
    let movement = if step < 20 { 
        0.0015 // Trending up (Delivery)
    } else if step < 35 {
        -0.0002 // Chopping (Distribution)
    } else {
        -0.0015 // Reversal (Reverse Period)
    };

    price += movement;

    Candle {
        timestamp: last.timestamp + Duration::hours(1),
        open: last.close,
        high: price.max(last.close) + 0.0002,
        low: price.min(last.close) - 0.0002,
        close: price,
        volume: 1000.0 + step as f64 * 10.0,
    }
}
