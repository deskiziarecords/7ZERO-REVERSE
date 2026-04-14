use crate::types::types::Candle;
use chrono::{DateTime, Utc};
use std::error::Error;
use std::path::Path;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct CsvRecord {
    timestamp: String,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

pub fn load_from_csv<P: AsRef<Path>>(path: P) -> Result<Vec<Candle>, Box<dyn Error>> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_path(path)?;

    let mut candles = Vec::new();
    for result in reader.deserialize() {
        let record: CsvRecord = result?;
        
        // Parse the timestamp. Format: 2025-04-14 21:00:00+00:00
        // We use DateTime::parse_from_str or similar.
        let timestamp = DateTime::parse_from_str(&record.timestamp, "%Y-%m-%d %H:%M:%S%z")?
            .with_timezone(&Utc);

        candles.push(Candle {
            timestamp,
            open: record.open,
            high: record.high,
            low: record.low,
            close: record.close,
            volume: record.volume,
        });
    }

    Ok(candles)
}
