# 7ZERO-REVERSE: IPDA REVERSE PERIOD TERMINAL

7ZERO-REVERSE is a high-frequency trading engine and visualization terminal designed to detect and execute "Reverse Period" protocols based on the Interbank Price Delivery Algorithm (IPDA). It combines a high-performance Rust core with a responsive React frontend, connected via WebAssembly (WASM).

## 🚀 Key Features

- **Core IPDA Logic**: Implements Phase Entrapment, Temporal Alignment, Spectral Inversion, and Liquidity Inversion detection.
- **Structural Detection**: Automatically identifies market regimes (Delivery, Consolidation, Reverse Period) using an Adelic Manifold Validator.
- **High-Performance Core**: Written in Rust for low-latency signal processing and mathematical validation.
- **WASM Integration**: Seamlessly bridges the Rust engine with a modern React/JavaScript frontend.
- **Interactive Terminal**: Real-time candlestick charting with structural range ("The Box") and equilibrium (Mean) visualization.

## 🏗️ Architecture

The system is split into two primary layers:

### 1. The Rust Core (`/src`)
- **`types.rs`**: Core data structures (Candle, Configuration, System States).
- **`core.rs`**: The `ReversePeriodEngine`, implementing the main update loop and transition logic.
- **`detectors.rs`**: The `DetectionMatrix`, which analyzes market data across 6 Lambda (λ) dimensions.
- **`math.rs`**: Low-level utilities for SMA, ATR, Standard Deviation, and Adelic mathematical validation.
- **`meta_cognitive.rs` & `neuro_symbolic.rs`**: Experimental layers for intent analysis and pattern recognition.

### 2. The React Frontend (`src/App.jsx`)
- Uses `lightweight-charts` for financial data visualization.
- Integrates the Rust engine via `@wasm-bindgen` for real-time tick-by-tick analysis.
- Provides a "Detection Matrix" dashboard to monitor λ-indicators and system health.

## 📊 The Detection Matrix (λ)

The system evaluates market state through six critical filters:
- **λ1: Phase Entrapment**: Detects prolonged distribution without expansion.
- **λ2: Temporal Alignment**: Monitors price movement within specific Killzones (London/NY).
- **λ3: Spectral Inversion**: Identifies divergence between price and momentum-based expectations.
- **λ4: Confluence Collapse**: Validates signal confidence against expected P&L.
- **λ5: Liquidity Inversion**: Analyzes gradient reversals in the liquidity field.
- **λ6: Displacement Veto**: A safety mechanism that halts the system during conflicting high-volatility events.

## 🛠️ Setup & Usage

### Prerequisites
- [Rust](https://rustup.rs/) (2021 edition)
- [Node.js](https://nodejs.org/) & `npm`
- [`wasm-pack`](https://rustwasm.github.io/wasm-pack/installer/)

### Backend (Rust)
To run a local market simulation and verify the engine logic:
```bash
cargo run
```

### Frontend (React + WASM)
To build the WASM package and start the development terminal:

1. Build the WASM package:
```bash
wasm-pack build
```

2. (Optional) Run frontend:
```bash
npm install
npm start
```

## 📜 License

Distributed under the MIT License. See `LICENSE` for more information.
