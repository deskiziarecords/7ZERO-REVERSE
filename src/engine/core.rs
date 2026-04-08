use crate::types::types::{Candle, DetectionMatrix, ReversePeriodConfig, SystemState, StructuralRange};
use crate::models::math::{calculate_sma, adelic_manifold_validator, calculate_atr};
use crate::engine::state::State;
use crate::engine::detectors::LambdaDetectors;

pub struct Engine {
    pub state: State,
    pub detectors: LambdaDetectors,
    pub system_state: SystemState,
    pub is_damped: bool,
}

impl Engine {
    pub fn new(config: ReversePeriodConfig) -> Self {
        Self {
            state: State::new(100),
            detectors: LambdaDetectors::new(config),
            system_state: SystemState::Delivery,
            is_damped: false,
        }
    }

    pub fn on_candle(&mut self, candle: Candle, current_volatility: f64, signal_confidence: f64, expected_pnl: f64) {
        self.state.push(candle.clone());
        let candles = self.state.get_candles();

        self.detectors.update(&candles, current_volatility, signal_confidence, expected_pnl);

        // Capture what we need from self before potentially calling a mutable method
        let (is_trigger_fired, matrix_lambda6_veto) = {
            let matrix = &self.detectors.last_matrix;

            if matrix.lambda6_displacement_veto {
                (false, true)
            } else {
                let r_score = self.calculate_severity_score(matrix);
                let last_candle = candles.last().unwrap();
                let is_coherent = adelic_manifold_validator(last_candle.close, current_volatility, last_candle.volume);
                let is_distribution = current_volatility < 0.001;
                let trigger_fired = is_coherent && is_distribution && (r_score > 0.6);
                (trigger_fired, false)
            }
        };

        if matrix_lambda6_veto {
            self.system_state = SystemState::Halted;
            return;
        }

        if is_trigger_fired {
            let matrix = self.detectors.last_matrix.clone();
            self.execute_reverse_period_protocol(&candles, &matrix);
        } else if self.system_state == SystemState::ReversePeriod {
            if let Some(range) = self.compile_structural_range() {
                let last_candle = candles.last().unwrap();
                if (last_candle.close - range.mean).abs() < (0.1 * (range.high - range.low)) {
                    self.system_state = SystemState::Delivery;
                }
            }
        } else if self.system_state == SystemState::Halted {
             self.system_state = SystemState::Delivery;
        }
    }

    pub fn calculate_severity_score(&self, matrix: &DetectionMatrix) -> f64 {
        let w = &self.detectors.config.lambda_weights;
        let mut score = 0.0;
        if matrix.lambda1_phase_entrapment { score += w.lambda1; }
        if matrix.lambda2_temporal_alignment { score += w.lambda2; }
        if matrix.lambda3_spectral_inversion { score += w.lambda3; }
        if matrix.lambda4_confluence_collapse { score += w.lambda4; }
        if matrix.lambda5_liquidity_inversion { score += w.lambda5; }
        score
    }

    pub fn compile_structural_range(&self) -> Option<StructuralRange> {
        let candles = self.state.get_candles();
        let lookback = self.detectors.config.lookback_60;
        if candles.len() < lookback {
            return None;
        }

        let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
        let highs: Vec<f64> = candles.iter().map(|c| c.high).collect();
        let lows: Vec<f64> = candles.iter().map(|c| c.low).collect();

        let l60_high = highs.iter().rev().take(lookback).cloned().fold(f64::NAN, f64::max);
        let l60_low = lows.iter().rev().take(lookback).cloned().fold(f64::NAN, f64::min);
        let l60_mean = calculate_sma(&closes, lookback)?;

        if l60_high.is_nan() || l60_low.is_nan() {
            return None;
        }

        Some(StructuralRange {
            high: l60_high,
            low: l60_low,
            mean: l60_mean,
        })
    }

    pub fn mandra_gate(&self, candles: &[Candle]) -> bool {
        if candles.len() < 2 { return false; }
        let curr = &candles[candles.len()-1];
        let prev = &candles[candles.len()-2];
        let e_curr = curr.volume * (curr.close - curr.open).abs().powi(2);
        let e_prev = prev.volume * (prev.close - prev.open).abs().powi(2);
        e_curr >= e_prev
    }

    fn execute_reverse_period_protocol(&mut self, candles: &[Candle], matrix: &DetectionMatrix) {
        let range = match self.compile_structural_range() {
            Some(r) => r,
            None => return,
        };

        let last = candles.last().unwrap();
        let atr = calculate_atr(candles, self.detectors.config.atr_period).unwrap_or(1.0);
        let target_price = range.mean;

        let wick_top = last.high - last.close;
        let wick_bottom = last.close - last.low;
        let is_sweep = (wick_top > 1.5 * atr || wick_bottom > 1.5 * atr) && (last.close >= range.low && last.close <= range.high);

        if !is_sweep && !matrix.lambda3_spectral_inversion {
            return;
        }

        let is_looping = candles.get(candles.len().saturating_sub(5))
            .map(|c5| (last.close - c5.close).abs() < atr)
            .unwrap_or(false);

        if is_looping {
            return;
        }

        if !self.mandra_gate(candles) {
            self.is_damped = true;
            return;
        }

        let distance = (last.close - target_price).abs();
        let period_bars = distance / atr;

        self.system_state = SystemState::ReversePeriod;
        self.is_damped = false;

        println!("=== REVERSE PERIOD TRIGGERED ===");
        println!("Target (Equilibrium): {}", target_price);
        println!("Est. Time to Target: {} bars", period_bars);
        println!("State: {:?}", self.system_state);
    }
}

pub struct ReversePeriodEngine {
    pub engine: Engine,
}

impl ReversePeriodEngine {
    pub fn new(config: ReversePeriodConfig) -> Self {
        Self {
            engine: Engine::new(config),
        }
    }

    pub fn update(&mut self, candles: &[Candle], current_volatility: f64, signal_confidence: f64) {
        if let Some(last) = candles.last() {
             self.engine.on_candle(last.clone(), current_volatility, signal_confidence, -100.0);
        }
    }

    pub fn system_state(&self) -> SystemState {
        self.engine.system_state
    }

    pub fn state(&self) -> SystemState {
        self.system_state()
    }

    pub fn is_damped(&self) -> bool {
        self.engine.is_damped
    }
}
