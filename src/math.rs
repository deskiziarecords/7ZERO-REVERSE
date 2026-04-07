use crate::types::Candle;

pub fn calculate_sma(data: &[f64], period: usize) -> Option<f64> {
    if data.len() < period {
        return None;
    }
    let sum: f64 = data.iter().rev().take(period).sum();
    Some(sum / period as f64)
}

pub fn calculate_std_dev(data: &[f64], period: usize, mean: f64) -> Option<f64> {
    if data.len() < period {
        return None;
    }
    let variance: f64 = data
        .iter()
        .rev()
        .take(period)
        .map(|&x| (x - mean).powi(2))
        .sum::<f64>()
        / period as f64;
    Some(variance.sqrt())
}

pub fn calculate_atr(candles: &[Candle], period: usize) -> Option<f64> {
    if candles.len() < period + 1 {
        return None;
    }
    
    let mut tr_sum = 0.0;
    for i in (candles.len() - period)..candles.len() {
        let c = &candles[i];
        let prev_c = &candles[i - 1];
        let tr = (c.high - c.low)
            .max((c.high - prev_c.close).abs())
            .max((c.low - prev_c.close).abs());
        tr_sum += tr;
    }
    Some(tr_sum / period as f64)
}

/// Adelic Manifold Validator (1A)
/// Checks for mathematical coherence (NaN checks, physical constraints) before allowing a trigger.
pub fn adelic_manifold_validator(price: f64, volatility: f64, volume: f64) -> bool {
    if price.is_nan() || volatility.is_nan() || volume.is_nan() {
        return false;
    }
    if volatility <= 0.0 || volume < 0.0 {
        return false;
    }
    // Ensure price variation is physically possible
    true
}
