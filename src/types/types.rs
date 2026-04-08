use chrono::{DateTime, Utc};

/// Represents a single candle of market data.
#[derive(Debug, Clone)]
pub struct Candle {
    pub timestamp: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

/// Configuration for the Reverse Period Engine
#[derive(Debug, Clone)]
pub struct ReversePeriodConfig {
    #[allow(dead_code)]
    pub lookback_20: usize,
    #[allow(dead_code)]
    pub lookback_40: usize,
    pub lookback_60: usize,
    pub atr_period: usize,
    pub killzone_london_start: u8, // Hour 0-23
    pub killzone_london_end: u8,
    pub killzone_ny_start: u8,
    pub killzone_ny_end: u8,
    pub lambda_weights: LambdaWeights,
}

/// Weights for the Rscore calculation
#[derive(Debug, Clone, Copy)]
pub struct LambdaWeights {
    pub lambda1: f64, // Phase Entrapment
    pub lambda2: f64, // Temporal Alignment
    pub lambda3: f64, // Spectral Inversion
    pub lambda4: f64, // Confluence Collapse
    pub lambda5: f64, // Liquidity Field
}

impl Default for LambdaWeights {
    fn default() -> Self {
        Self {
            lambda1: 0.35,
            lambda2: 0.25,
            lambda3: 0.20,
            lambda4: 0.15,
            lambda5: 0.05,
        }
    }
}

/// The Core Detection Matrix results
#[derive(Debug, Clone, Copy, Default)]
pub struct DetectionMatrix {
    pub lambda1_phase_entrapment: bool,
    pub lambda2_temporal_alignment: bool,
    pub lambda3_spectral_inversion: bool,
    pub lambda4_confluence_collapse: bool,
    pub lambda5_liquidity_inversion: bool,
    pub lambda6_displacement_veto: bool, // Veto (True means HALT)
}

/// Current State of the 7 ZERO System
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SystemState {
    Delivery,    // Trending phase
    #[allow(dead_code)]
    Consolidation, // Chop/Range
    ReversePeriod, // Active Reversal Trade
    Halted,      // Veto or System Error
}

/// Structural Range Data ("The Box")
#[derive(Debug, Clone, Copy)]
pub struct StructuralRange {
    pub high: f64,
    pub low: f64,
    pub mean: f64, // Equilibrium Point (Value)
}
