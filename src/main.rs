pub mod types;
pub mod data;
pub mod engine;
pub mod models;

use chrono::{Duration, Utc};
use types::types::{Candle, ReversePeriodConfig, LambdaWeights};
use engine::core::ReversePeriodEngine;

use data::bitget::BitgetProvider;
use data::csv_loader::load_from_csv;
use data::metatrader::MetaTraderProvider;
use types::config::BrokerCredentials;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load Credentials from .env
    let creds = BrokerCredentials::load_from_env();

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

    // Data Selection (Hardcoded for demonstration, could be CLI args)
    // Mode options: "MOCK", "BITGET", "CSV", "XM"
    let mode = "MOCK"; 

    let mut candles = match mode {
        "BITGET" => {
            println!("Fetching live data from Bitget...");
            let provider = BitgetProvider::new("BTCUSDT", "1m");
            provider.fetch_recent_candles(100).await?
        },
        "CSV" => {
            println!("Loading historical data from CSV...");
            load_from_csv("data/eur_usd_2024.csv")?
        },
        "XM" | "METATRADER" => {
            println!("Connecting to XM/MetaTrader...");
            if let Some(mt_creds) = creds.xm {
                let provider = MetaTraderProvider::new(mt_creds, "EURUSD");
                provider.fetch_recent_candles(100).await?
            } else {
                println!("Error: XM Credentials not found in .env");
                return Ok(());
            }
        },
        _ => {
            println!("Running in MOCK mode...");
            generate_mock_data(100)
        }
    };

    println!("Initial processing of {} candles...", candles.len());
    for candle in &candles {
        engine.update(&vec![candle.clone()], 0.001, 0.5);
    }

    // Process future ticks/candles if applicable
    if mode == "MOCK" {
        for i in 0..50 {
            let new_candle = create_next_candle(&candles, i);
            candles.push(new_candle.clone());
            engine.update(&candles, 0.0005, 0.8);
            print_state(&engine, i + 101);
        }
    } else {
        println!("Completed processing. Current State: {:?}", engine.system_state());
    }

    Ok(())
}

fn print_state(engine: &ReversePeriodEngine, bar_idx: usize) {
    let state = engine.system_state();
    let r_score = {
        let matrix = &engine.engine.detectors.last_matrix;
        engine.engine.calculate_severity_score(matrix)
    };
    println!("Bar {}: State={:?}, R-Score={:.2}, Damped={}", bar_idx, state, r_score, engine.is_damped());
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
