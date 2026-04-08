use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{Duration, Utc};
use rand::Rng;

// Import local modules (adjust paths if your structure differs)
use crate::types::{Candle, DetectionMatrix, SystemState, ReversePeriodConfig, LambdaWeights, StructuralRange};
use crate::core::ReversePeriodEngine;
use crate::detectors;
use crate::math;

// --- 1. Serializable Structures for JS ---

/// Simplified Candle for JS (Dates as i64 timestamps to avoid WASM complexity)
#[derive(Serialize, Deserialize, Clone)]
pub struct JsCandle {
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

impl From<Candle> for JsCandle {
    fn from(c: Candle) -> Self {
        Self {
            timestamp: c.timestamp.timestamp_millis(),
            open: c.open,
            high: c.high,
            low: c.low,
            close: c.close,
            volume: c.volume,
        }
    }
}

/// Output state sent to the Frontend
#[derive(Serialize, Deserialize)]
pub struct EngineOutput {
    pub state: String,           // "Delivery", "ReversePeriod", etc.
    pub r_score: f64,            // 0.0 to 1.0
    pub matrix: DetectionMatrixJS, // The Lambda Booleans
    pub range: StructuralRangeJS, // The Box (High, Low, Mean)
    pub target: f64,             // Equilibrium (Same as Mean)
}

/// JS-friendly version of DetectionMatrix
#[derive(Serialize, Deserialize)]
pub struct DetectionMatrixJS {
    pub l1: bool, // Phase Entrapment
    pub l2: bool, // Temporal Alignment
    pub l3: bool, // Spectral Inversion
    pub l4: bool, // Confluence Collapse
    pub l5: bool, // Liquidity Inversion
    pub l6: bool, // Displacement Veto
}

/// JS-friendly version of StructuralRange
#[derive(Serialize, Deserialize)]
pub struct StructuralRangeJS {
    pub high: f64,
    pub low: f64,
    pub mean: f64,
}

// --- 2. The WASM Wrapper ---

#[wasm_bindgen]
pub struct WasmEngine {
    engine: ReversePeriodEngine,
    candles: Vec<Candle>,
    config: ReversePeriodConfig,
}

#[wasm_bindgen]
impl WasmEngine {
    /// Constructor: Initializes the 7 ZERO engine and seeds initial data
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmEngine {
        // 1. Setup Configuration
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

        // 2. Initialize Engine
        let engine = ReversePeriodEngine::new(config.clone());

        // 3. Generate Seed Data (Mock Historical Data)
        let candles = Self::generate_mock_history(100);

        WasmEngine {
            engine,
            candles,
            config,
        }
    }

    /// Main Loop: Called by JS to advance time and price
    /// price_delta: The simulated price movement for this tick
    #[wasm_bindgen]
    pub fn tick(&mut self, price_delta: f64) -> JsValue {
        // 1. Create new candle based on last close + delta
        let last_candle = self.candles.last().unwrap();
        let new_close = last_candle.close + price_delta;
        
        // Mocking Open/High/Low based on the close
        let body_size = (new_close - last_candle.close).abs();
        let noise = (rand::thread_rng().gen::<f64>() - 0.5) * 0.0001;
        
        let new_candle = Candle {
            timestamp: Utc::now(),
            open: last_candle.close,
            high: last_candle.close.max(new_close) + noise.abs(),
            low: last_candle.close.min(new_close) - noise.abs(),
            close: new_close,
            volume: 1000.0 + (rand::thread_rng().gen::<f64>() * 500.0),
        };

        self.candles.push(new_candle);

        // 2. Run the Core Logic
        // Calculate mock volatility and confidence for the simulation
        let atr = math::calculate_atr(&self.candles, 14).unwrap_or(0.0001);
        let current_vol = (new_candle.high - new_candle.low) / new_candle.close;
        
        // Confidence increases if we are near edges of range (Mock logic)
        let range_high = self.candles.iter().rev().take(60).map(|c| c.high).fold(f64::NAN, f64::max);
        let range_low = self.candles.iter().rev().take(60).map(|c| c.low).fold(f64::NAN, f64::min);
        let position = (new_candle.close - range_low) / (range_high - range_low);
        let confidence = if position > 0.9 || position < 0.1 { 0.8 } else { 0.3 };

        self.engine.update(&self.candles, current_vol, confidence);

        // 3. Gather State for UI
        // Note: We re-calculate matrix and range here because the engine
        // doesn't necessarily store them as public fields in the previous step.
        let matrix = detectors::DetectionMatrix::analyze(
            &self.candles, 
            &self.config, 
            current_vol, 
            -50.0, // Mock PnL
            confidence
        );

        let range = self.calculate_range();
        let r_score = self.calculate_score(&matrix);

        // 4. Serialize and Return
        let output = EngineOutput {
            state: format!("{:?}", self.engine.state),
            r_score,
            matrix: DetectionMatrixJS {
                l1: matrix.lambda1_phase_entrapment,
                l2: matrix.lambda2_temporal_alignment,
                l3: matrix.lambda3_spectral_inversion,
                l4: matrix.lambda4_confluence_collapse,
                l5: matrix.lambda5_liquidity_inversion,
                l6: matrix.lambda6_displacement_veto,
            },
            range: StructuralRangeJS {
                high: range.high,
                low: range.low,
                mean: range.mean,
            },
            target: range.mean,
        };

        serde_wasm_bindgen::to_value(&output).unwrap()
    }

    /// Returns the full candle history for the chart
    #[wasm_bindgen]
    pub fn get_candles(&self) -> JsValue {
        let js_candles: Vec<JsCandle> = self.candles.iter().cloned().map(|c| c.into()).collect();
        serde_wasm_bindgen::to_value(&js_candles).unwrap()
    }

    // --- Helper Methods ---

    fn generate_mock_history(count: usize) -> Vec<Candle> {
        let mut data = Vec::new();
        let now = Utc::now();
        let mut price = 1.1000;
        let mut rng = rand::thread_rng();

        for i in 0..count {
            let noise = (rng.gen::<f64>() - 0.5) * 0.0010;
            price += noise;

            data.push(Candle {
                timestamp: now - Duration::hours((count - i) as i64),
                open: price,
                high: price + 0.0005,
                low: price - 0.0005,
                close: price + (rng.gen::<f64>() - 0.5) * 0.0002,
                volume: 1000.0,
            });
        }
        data
    }

    fn calculate_range(&self) -> StructuralRange {
        // Simplified range calculation matching core logic
        let closes: Vec<f64> = self.candles.iter().map(|c| c.close).collect();
        let highs: Vec<f64> = self.candles.iter().map(|c| c.high).collect();
        let lows: Vec<f64> = self.candles.iter().map(|c| c.low).collect();

        let lookback = self.config.lookback_60;
        if self.candles.len() < lookback {
             return StructuralRange { high: 0.0, low: 0.0, mean: 0.0 };
        }

        let slice_high = &highs[self.candles.len() - lookback..];
        let slice_low = &lows[self.candles.len() - lookback..];
        
        let high = slice_high.iter().cloned().fold(f64::NAN, f64::max);
        let low = slice_low.iter().cloned().fold(f64::NAN, f64::min);
        
        // Calculate mean of closes
        let sum: f64 = closes.iter().rev().take(lookback).sum();
        let mean = sum / lookback as f64;

        StructuralRange { high, low, mean }
    }

    fn calculate_score(&self, matrix: &DetectionMatrix) -> f64 {
        let w = &self.config.lambda_weights;
        let mut score = 0.0;
        if matrix.lambda1_phase_entrapment { score += w.lambda1; }
        if matrix.lambda2_temporal_alignment { score += w.lambda2; }
        if matrix.lambda3_spectral_inversion { score += w.lambda3; }
        if matrix.lambda4_confluence_collapse { score += w.lambda4; }
        if matrix.lambda5_liquidity_inversion { score += w.lambda5; }
        score
    }
}


// ... existing imports
mod meta_cognitive;
mod neuro_symbolic; // The stub module

use meta_cognitive::{MetaCognitiveToolSelector, IPDAMetaCognitiveServer};

// ... existing WasmEngine implementation ...

// --- NEW COGNITIVE SERVER WRAPPER ---

#[wasm_bindgen]
pub struct CognitiveServer {
    selector: MetaCognitiveToolSelector,
}

#[wasm_bindgen]
impl CognitiveServer {
    #[wasm_bindgen(constructor)]
    pub fn new() -> CognitiveServer {
        CognitiveServer {
            selector: MetaCognitiveToolSelector::new(),
        }
    }

    /// Process a natural language query through the Neuro-Symbolic engine
    #[wasm_bindgen]
    pub fn ask(&mut self, query: &str) -> JsValue {
        // 1. Analyze Intent
        let intent = self.selector.analyze_intent(query);
        
        // 2. Generate Plan
        let plan = self.selector.generate_tool_plan(&intent);
        
        // 3. Execute Plan
        let execution = self.selector.execute_plan(&plan);
        
        // 4. Reflect
        self.selector.reflect_and_learn(&execution, query);

        // 5. Prepare Output for Frontend
        let response = if execution.final_confidence > 0.7 {
            format!("High confidence result from {} tool calls.", execution.results.len())
        } else if execution.final_confidence > 0.4 {
            "Moderate confidence. Consider additional validation.".to_string()
        } else {
            format!("Low confidence ({:.0}%). Recommend manual review.", execution.final_confidence * 100.0)
        };

        let output = serde_json::json!({
            "query": query,
            "intent_type": intent.intent_type,
            "plan": plan,
            "confidence": execution.final_confidence,
            "response": response,
            "trace": execution.results
        });

        serde_wasm_bindgen::to_value(&output).unwrap()
    }
}
