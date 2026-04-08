use chrono::Timelike;
use crate::models::math::{calculate_atr, calculate_sma};
use crate::types::types::{Candle, DetectionMatrix, ReversePeriodConfig};

pub struct LambdaDetectors {
    pub config: ReversePeriodConfig,
    pub last_matrix: DetectionMatrix,
}

impl LambdaDetectors {
    pub fn new(config: ReversePeriodConfig) -> Self {
        Self {
            config,
            last_matrix: DetectionMatrix::default(),
        }
    }

    pub fn update(&mut self, candles: &[Candle], volatility: f64, confidence: f64, expected_pnl: f64) {
        if candles.is_empty() {
            return;
        }

        // We use the existing analyze logic
        self.last_matrix = DetectionMatrix::analyze(
            candles,
            &self.config,
            volatility,
            expected_pnl,
            confidence,
        );
    }

    pub fn verify_institutional_dna(&self) -> bool {
        let w = &self.config.lambda_weights;
        let mut score = 0.0;
        let matrix = &self.last_matrix;

        if matrix.lambda1_phase_entrapment { score += w.lambda1; }
        if matrix.lambda2_temporal_alignment { score += w.lambda2; }
        if matrix.lambda3_spectral_inversion { score += w.lambda3; }
        if matrix.lambda4_confluence_collapse { score += w.lambda4; }
        if matrix.lambda5_liquidity_inversion { score += w.lambda5; }

        score > 0.6
    }
}

impl DetectionMatrix {
    /// Runs the full Core Detection Matrix
    pub fn analyze(
        candles: &[Candle],
        cfg: &ReversePeriodConfig,
        current_volatility: f64,
        expected_pnl: f64,
        signal_confidence: f64,
    ) -> Self {
        if candles.len() < 2 {
            return Self::default();
        }
        let atr = calculate_atr(candles, cfg.atr_period).unwrap_or(0.0);
        let current_candle = candles.last().unwrap();
        let prev_candle = candles.get(candles.len() - 2);

        // Lambda 1: Phase Entrapment
        // Prolonged distribution (sigma=2 proxy) without expansion
        let range_size = current_candle.high - current_candle.low;
        let is_low_volatility = range_size < (0.5 * atr); // Proxy for sigma=2 distribution
        let lambda1 = is_low_volatility && (current_volatility > 0.0 && current_volatility < 0.002); // tight spread

        // Lambda 2: Temporal Alignment Failure
        // Price fails to move in Killzone
        let hour = current_candle.timestamp.hour() as u8;
        let is_kz = (hour >= cfg.killzone_london_start && hour <= cfg.killzone_london_end) ||
                    (hour >= cfg.killzone_ny_start && hour <= cfg.killzone_ny_end);
        
        let move_size = (current_candle.close - current_candle.open).abs();
        let lambda2 = is_kz && (move_size < (0.2 * atr)); // Failed to move

        // Lambda 3: Spectral Inversion
        // Phase difference > 90 degrees between predicted (SMA 20) and actual
        let sma_20 = calculate_sma(&candles.iter().map(|c| c.close).collect::<Vec<_>>(), 20);
        let lambda3 = if let Some(sma) = sma_20 {
            // Simple harmonic proxy: Price moves away from mean while momentum says it should revert
            let displacement = current_candle.close - sma;
            let divergence = displacement.abs() > (1.5 * atr);
            divergence
        } else {
            false
        };

        // Lambda 4: Confluence Collapse
        // High confidence but negative expected P&L
        let lambda4 = signal_confidence > 0.6 && expected_pnl < 0.0;

        // Lambda 5: Liquidity Field Inversion
        // Gradient reversal (Dot product < 0)
        let lambda5 = if let Some(prev) = prev_candle {
            let grad_curr = current_candle.close - current_candle.open;
            let grad_hist = (prev.close - prev.open) + (candles.get(candles.len().saturating_sub(3)).map(|c| c.close - c.open).unwrap_or(0.0));
            grad_curr * grad_hist < 0.0
        } else {
            false
        };

        // Lambda 6: Displacement Veto
        // 70% body ratio conflict
        let body_size = (current_candle.close - current_candle.open).abs();
        let candle_range = current_candle.high - current_candle.low;
        let is_large_body = candle_range > 0.0 && (body_size / candle_range) > 0.70;
        
        // Assuming macro intent is Short (Delivery exhausted at high), but candle is Huge Bullish
        // We determine macro intent by where we are in the range (simplified here as high close)
        let is_at_high = current_candle.close > current_candle.open; 
        let lambda6 = is_large_body && is_at_high; // Veto triggers if conflict exists

        DetectionMatrix {
            lambda1_phase_entrapment: lambda1,
            lambda2_temporal_alignment: lambda2,
            lambda3_spectral_inversion: lambda3,
            lambda4_confluence_collapse: lambda4,
            lambda5_liquidity_inversion: lambda5,
            lambda6_displacement_veto: lambda6,
        }
    }
}
