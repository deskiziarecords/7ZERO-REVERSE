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
        // v2 mix candles require productType (usdt-futures, coin-futures, etc.)
        let url = format!(
            "https://api.bitget.com/api/v2/mix/market/candles?symbol={}&granularity={}&limit={}&productType=usdt-futures",
            self.symbol, self.interval, limit
        );

        let response = reqwest::get(url).await?.json::<serde_json::Value>().await?;
        
        let data = response["data"].as_array().ok_or("Invalid response from Bitget: 'data' field missing or not an array")?;

        let mut candles = Vec::new();
        for item in data {
            let row = item.as_array().ok_or("Invalid candle item in data array")?;
            if row.len() < 6 { continue; }

            // Robust parsing: handles both strings (API default) and potential numeric types
            let ts_ms = if let Some(s) = row[0].as_str() {
                s.parse::<i64>().map_err(|_| "Failed to parse timestamp string")?
            } else if let Some(i) = row[0].as_i64() {
                i
            } else {
                return Err("Invalid timestamp type".into());
            };

            let parse_f64 = |v: &serde_json::Value| -> Result<f64, String> {
                v.as_str()
                    .map(|s| s.parse::<f64>().map_err(|e| e.to_string()))
                    .unwrap_or_else(|| v.as_f64().ok_or_else(|| "Not a valid number".to_string()))
            };

            let open   = parse_f64(&row[1]).map_err(|e| format!("Open parse error: {}", e))?;
            let high   = parse_f64(&row[2]).map_err(|e| format!("High parse error: {}", e))?;
            let low    = parse_f64(&row[3]).map_err(|e| format!("Low parse error: {}", e))?;
            let close  = parse_f64(&row[4]).map_err(|e| format!("Close parse error: {}", e))?;
            let volume = parse_f64(&row[5]).map_err(|e| format!("Volume parse error: {}", e))?;

            candles.push(Candle {
                timestamp: Utc.timestamp_millis_opt(ts_ms).single().ok_or("Invalid timestamp value")?,
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
