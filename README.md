# 7ZERO-REVERSE: IPDA REVERSE PERIOD TERMINAL

7ZERO-REVERSE is a high-performance trading engine and visualization terminal designed to detect and execute "Reverse Period" protocols based on the Interbank Price Delivery Algorithm (IPDA). It combines a modular Rust backend with a responsive React frontend, connected via WebAssembly (WASM).

##  Key Features

- **Core IPDA Logic**: Implements Phase Entrapment, Temporal Alignment, Spectral Inversion, and Liquidity Inversion detection via the Lambda Detection Matrix.
- **Real-time Data Processing**: Integrated layers for OANDA tick streaming and automated candle building.
- **Event-Driven Architecture**: High-performance engine orchestrating state management and signal detection on a per-candle basis.
- **Structural Detection**: Automatically identifies market regimes (Delivery, Consolidation, Reverse Period) using an Adelic Manifold Validator.
- **WASM Integration**: Seamlessly bridges the Rust engine with a modern React/JavaScript frontend for real-time visualization.

##  Architecture

The Rust backend is organized into a modular structure designed for scalability and low-latency processing:

### 1. Data Layer (`/src/data`)
- **`realtime.rs`**: Handles connections to market data providers (e.g., OANDA) and streams raw price ticks.
- **`candle_builder.rs`**: Aggregates raw ticks into standard interval candles (e.g., 1-minute).

### 2. Engine Layer (`/src/engine`)
- **`core.rs`**: The central `Engine` orchestrator, managing system state transitions and signal execution.
- **`detectors.rs`**: Implements the `LambdaDetectors` and `DetectionMatrix` for institutional DNA verification.
- **`state.rs`**: Manages a rolling window of market data using optimized state containers.

### 3. Models Layer (`/src/models`)
- **`math.rs`**: Low-level mathematical utilities (SMA, ATR, Standard Deviation) and the Adelic Manifold Validator.
- **`meta_cognitive.rs` & `neuro_symbolic.rs`**: Experimental layers for query intent analysis and pattern recognition.

### 4. Types Layer (`/src/types`)
- **`types.rs`**: Centralized definition of core data structures, configurations, and system states.

### 5. Frontend (`/frontend` & `src/App.jsx`)
- Uses `lightweight-charts` for financial data visualization.
- Integrates the Rust engine via `@wasm-bindgen` for real-time tick-by-tick analysis.
- Provides a comprehensive dashboard to monitor Lambda indicators and engine health.

##  The Detection Matrix (λ)

The system evaluates market state through six critical filters:
- **λ1: Phase Entrapment**: Detects prolonged distribution without expansion.
- **λ2: Temporal Alignment**: Monitors price movement within specific Killzones (London/NY).
- **λ3: Spectral Inversion**: Identifies divergence between price and momentum-based expectations.
- **λ4: Confluence Collapse**: Validates signal confidence against expected P&L.
- **λ5: Liquidity Inversion**: Analyzes gradient reversals in the liquidity field.
- **λ6: Displacement Veto**: A safety mechanism that halts the system during conflicting high-volatility events.

##  Setup & Usage

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
cd frontend
npm install
npm run dev
```

##  License

Distributed under the MIT License. See `LICENSE` for more information.
