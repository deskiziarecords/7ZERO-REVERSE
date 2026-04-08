use crate::types::{Candle, DetectionMatrix, ReversePeriodConfig, SystemState, StructuralRange};
use crate::math::{calculate_sma, adelic_manifold_validator};

pub struct ReversePeriodEngine {
    pub config: ReversePeriodConfig,
    pub state: SystemState,
    pub is_damped: bool, // Adelic-KL Damping
}

impl ReversePeriodEngine {
    pub fn new(config: ReversePeriodConfig) -> Self {
        Self {
            config,
            state: SystemState::Delivery,
            is_damped: false,
        }
    }

    /// Step 1: Define Structural Range (Compiler)
    fn compile_structural_range(&self, candles: &[Candle]) -> Option<StructuralRange> {
        let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
        let highs: Vec<f64> = candles.iter().map(|c| c.high).collect();
        let lows: Vec<f64> = candles.iter().map(|c| c.low).collect();

        // Synchronize 20, 40, 60 day lookbacks. We use L60 as the structural "Box".
        let l60_high = highs.iter().rev().take(self.config.lookback_60).cloned().fold(f64::NAN, f64::max);
        let l60_low = lows.iter().rev().take(self.config.lookback_60).cloned().fold(f64::NAN, f64::min);
        let l60_mean = calculate_sma(&closes, self.config.lookback_60)?;

        if l60_high.is_nan() || l60_low.is_nan() {
            return None;
        }

        Some(StructuralRange {
            high: l60_high,
            low: l60_low,
            mean: l60_mean,
        })
    }

    /// Calculates the Unified Reverse Period Score
    fn calculate_severity_score(&self, matrix: &DetectionMatrix) -> f64 {
        let w = &self.config.lambda_weights;
        let mut score = 0.0;

        if matrix.lambda1_phase_entrapment { score += w.lambda1; }
        if matrix.lambda2_temporal_alignment { score += w.lambda2; }
        if matrix.lambda3_spectral_inversion { score += w.lambda3; }
        if matrix.lambda4_confluence_collapse { score += w.lambda4; }
        if matrix.lambda5_liquidity_inversion { score += w.lambda5; }

        score
    }

    /// Bit Second Law (Mandra Gate)
    /// Ensures dE/dt >= 0 (Energy is non-decreasing)
    fn mandra_gate(&self, candles: &[Candle]) -> bool {
        if candles.len() < 2 { return false; }
        
        let curr = &candles[candles.len()-1];
        let prev = &candles[candles.len()-2];

        // Kinetic Energy proxy: Volume * (Price Change)^2
        let e_curr = curr.volume * (curr.close - curr.open).abs().powi(2);
        let e_prev = prev.volume * (prev.close - prev.open).abs().powi(2);

        e_curr >= e_prev
    }

    /// Main Update Loop
    pub fn update(&mut self, candles: &[Candle], current_volatility: f64, signal_confidence: f64) {
        if candles.len() < self.config.lookback_60 {
            return;
        }

        // 1. Run Detectors
        let matrix = DetectionMatrix::analyze(
            candles,
            &self.config,
            current_volatility,
            -100.0, // Mock PnL input
            signal_confidence
        );

        // 2. Check Veto (Lambda 6)
        if matrix.lambda6_displacement_veto {
            self.state = SystemState::Halted;
            return;
        }

        // 3. Calculate Severity Score
        let r_score = self.calculate_severity_score(&matrix);

        // Adelic Manifold Validator (1A) Check
        let last_candle = candles.last().unwrap();
        let is_coherent = adelic_manifold_validator(last_candle.close, current_volatility, last_candle.volume);

        // Trigger Logic: Rt = 1A * [sigma=2] * [max(...) > theta]
        // Assuming current_volatility < threshold implies [sigma=2] (distribution) vs expansion
        let is_distribution = current_volatility < 0.001; 
        let is_trigger_fired = is_coherent && is_distribution && (r_score > 0.6);

        if is_trigger_fired {
            // Transition to Reverse Protocol
            self.execute_reverse_period_protocol(candles, &matrix);
        } else if self.state == SystemState::ReversePeriod {
            // Check if we returned to value (Mean)
            let range = self.compile_structural_range(candles).unwrap();
            if (last_candle.close - range.mean).abs() < (0.1 * (range.high - range.low)) {
                self.state = SystemState::Delivery; // Reset to Delivery
            }
        }
    }

    /// Step-by-Step Calculation Protocol for Execution
    fn execute_reverse_period_protocol(&mut self, candles: &[Candle], matrix: &DetectionMatrix) {
        let range = match self.compile_structural_range(candles) {
            Some(r) => r,
            None => return,
        };

        let last = candles.last().unwrap();
        let atr = crate::math::calculate_atr(candles, self.config.atr_period).unwrap_or(1.0);

        // Step: Identify the Magnet (Target = L60 Mean)
        let target_price = range.mean;

        // Step: Detect Delivery Exhaustion (Layer 4 Sweep)
        // Logic: Wick > 1.5 * ATR and close back inside range
        let wick_top = last.high - last.close;
        let wick_bottom = last.close - last.low;
        let is_sweep = (wick_top > 1.5 * atr || wick_bottom > 1.5 * atr) && (last.close >= range.low && last.close <= range.high);

        if !is_sweep && !matrix.lambda3_spectral_inversion {
            return; // Invalid trigger
        }

        // Step: Metacognitive Verification
        // Ensure no TDA loops (Mock: check if last 5 candles are ranging)
        let is_looping = (candles[candles.len()-1].close - candles[candles.len()-5].close).abs() < atr;
        if is_looping {
            return; // Ranging trap
        }

        // Step: Gated Execution (Mandra Gate)
        if !self.mandra_gate(candles) {
            // Energy violation, halt
            self.is_damped = true; // Apply Logical Viscosity
            return;
        }

        // Step: Project Temporal Window
        let distance = (last.close - target_price).abs();
        let period_bars = distance / atr;

        // If we get here, State is valid Reverse Period
        self.state = SystemState::ReversePeriod;
        self.is_damped = false; // Unfreeze if valid

        println!("=== REVERSE PERIOD TRIGGERED ===");
        println!("Target (Equilibrium): {}", target_price);
        println!("Est. Time to Target: {} bars", period_bars);
        println!("State: {:?}", self.state);
    }
}
