use crate::types::types::Candle;
use chrono::{Utc, TimeZone};
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct BitgetCandleResponse(pub Vec<Vec<String>>);

pub struct BitgetProvider {
    pub symbol: String, // e.g. "BTCUSDT"
    pub interval: String, // e.g. "1min", "5min", "1h"
}

impl BitgetProvider {
    pub fn new(symbol: &str, interval: &str) -> Self {
        Self {
            symbol: symbol.to_string(),
            interval: interval.to_string(),
        }
    }

    pub async fn fetch_recent_candles(&self, limit: usize) -> Result<Vec<Candle>, Box<dyn Error>> {
        let url = format!(
            "https://api.bitget.com/api/v2/mix/market/candles?symbol={}&granularity={}&limit={}",
            self.symbol, self.interval, limit
        );

        let response = reqwest::get(url).await?.json::<serde_json::Value>().await?;
        
        // Bitget v2 response format for candles is usually in ["data"]
        let data = response["data"].as_array().ok_or("Invalid response from Bitget")?;

        let mut candles = Vec::new();
        for item in data {
            let row = item.as_array().ok_or("Invalid candle item")?;
            if row.len() < 6 { continue; }

            let ts_ms = row[0].as_str().ok_or("Invalid timestamp")?.parse::<i64>()?;
            let open = row[1].as_str().ok_or("Invalid open")?.parse::<f64>()?;
            let high = row[2].as_str().ok_or("Invalid high")?.parse::<f64>()?;
            let low = row[3].as_str().ok_or("Invalid low")?.parse::<f64>()?;
            let close = row[4].as_str().ok_or("Invalid close")?.parse::<f64>()?;
            let volume = row[5].as_str().ok_or("Invalid volume")?.parse::<f64>()?;

            candles.push(Candle {
                timestamp: Utc.timestamp_millis_opt(ts_ms).unwrap(),
                open,
                high,
                low,
                close,
                volume,
            });
        }

        // Bitget returns candles in reverse chronological order (newest first), we want oldest first
        candles.reverse();
        Ok(candles)
    }
}
