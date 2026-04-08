use crate::types::types::Candle;
use chrono::{DateTime, Utc, Timelike};

pub struct CandleBuilder {
    current_candle: Option<Candle>,
    interval_minutes: u32,
}

impl CandleBuilder {
    pub fn new(interval_minutes: u32) -> Self {
        Self {
            current_candle: None,
            interval_minutes,
        }
    }

    /// Converts ticks into candles
    pub fn process_tick(&mut self, price: f64, timestamp: DateTime<Utc>) -> Option<Candle> {
        let rounded_time = self.round_time(timestamp);

        if let Some(mut candle) = self.current_candle.take() {
            if rounded_time > candle.timestamp {
                let completed_candle = candle.clone();
                self.current_candle = Some(Candle {
                    timestamp: rounded_time,
                    open: price,
                    high: price,
                    low: price,
                    close: price,
                    volume: 1.0,
                });
                return Some(completed_candle);
            } else {
                candle.high = candle.high.max(price);
                candle.low = candle.low.min(price);
                candle.close = price;
                candle.volume += 1.0;
                self.current_candle = Some(candle);
                return None;
            }
        } else {
            self.current_candle = Some(Candle {
                timestamp: rounded_time,
                open: price,
                high: price,
                low: price,
                close: price,
                volume: 1.0,
            });
            return None;
        }
    }

    fn round_time(&self, ts: DateTime<Utc>) -> DateTime<Utc> {
        let minute = (ts.minute() / self.interval_minutes) * self.interval_minutes;
        ts.with_minute(minute).unwrap().with_second(0).unwrap().with_nanosecond(0).unwrap()
    }
}
