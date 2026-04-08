/**
 * 7ZERO-REVERSE: Pure JS Engine (Simulation Mode)
 * Mirrors the Rust ReversePeriodEngine logic exactly.
 * Swap this out for the WASM bindings once wasm-pack build is complete.
 */

// ── Math Utilities ────────────────────────────────────────────────────────────

export function calculateSMA(closes, period) {
  if (closes.length < period) return null;
  const slice = closes.slice(-period);
  return slice.reduce((a, b) => a + b, 0) / period;
}

export function calculateATR(candles, period = 14) {
  if (candles.length < period + 1) return null;
  const trs = [];
  for (let i = 1; i < candles.length; i++) {
    const c = candles[i];
    const p = candles[i - 1];
    trs.push(Math.max(c.high - c.low, Math.abs(c.high - p.close), Math.abs(c.low - p.close)));
  }
  const recent = trs.slice(-period);
  return recent.reduce((a, b) => a + b, 0) / recent.length;
}

export function calculateSTD(values, period) {
  if (values.length < period) return 0;
  const slice = values.slice(-period);
  const mean = slice.reduce((a, b) => a + b, 0) / period;
  const variance = slice.reduce((s, v) => s + (v - mean) ** 2, 0) / period;
  return Math.sqrt(variance);
}

// Adelic Manifold Validator: ensures price coherence across p-adic fields
export function adelicManifoldValidator(price, volatility, volume) {
  if (price <= 0 || volume <= 0) return false;
  const p2 = Math.abs(price * 2) % 1;   // 2-adic residue proxy
  const p3 = Math.abs(price * 3) % 1;   // 3-adic residue proxy
  const coherence = 1 - Math.abs(p2 - p3);
  return coherence > 0.3 && volatility < 0.005;
}

// ── Detection Matrix ──────────────────────────────────────────────────────────

export function analyzeDetectionMatrix(candles, cfg, currentVolatility, expectedPnL, signalConfidence) {
  if (candles.length < 5) return nullMatrix();

  const cur = candles[candles.length - 1];
  const prev = candles[candles.length - 2];
  const prev2 = candles[candles.length - 3];
  const atr = calculateATR(candles, cfg.atrPeriod) ?? 0.0001;
  const closes = candles.map(c => c.close);

  // λ1: Phase Entrapment — tight range, low Vol, sigma=2 distribution
  const rangeSize = cur.high - cur.low;
  const isLowVol = rangeSize < 0.5 * atr;
  const l1 = isLowVol && currentVolatility > 0 && currentVolatility < 0.002;

  // λ2: Temporal Alignment — price fails to move during Killzone hours
  const hour = new Date(cur.timestamp).getUTCHours();
  const inLondon = hour >= cfg.killzoneLondonStart && hour <= cfg.killzoneLondonEnd;
  const inNY     = hour >= cfg.killzoneNYStart     && hour <= cfg.killzoneNYEnd;
  const isKZ = inLondon || inNY;
  const moveSize = Math.abs(cur.close - cur.open);
  const l2 = isKZ && moveSize < 0.2 * atr;

  // λ3: Spectral Inversion — divergence between SMA-20 and actual price
  const sma20 = calculateSMA(closes, 20);
  let l3 = false;
  if (sma20 !== null) {
    const displacement = cur.close - sma20;
    l3 = Math.abs(displacement) > 1.5 * atr;
  }

  // λ4: Confluence Collapse — high confidence but negative expected PnL
  const l4 = signalConfidence > 0.6 && expectedPnL < 0;

  // λ5: Liquidity Field Inversion — gradient reversal (dot product < 0)
  const gradCurr = cur.close - cur.open;
  const gradHist = (prev.close - prev.open) + (prev2.close - prev2.open);
  const l5 = gradCurr * gradHist < 0;

  // λ6: Displacement Veto — large bullish body vs bearish delivery intent
  const bodySize  = Math.abs(cur.close - cur.open);
  const fullRange = cur.high - cur.low;
  const isLargeBody = fullRange > 0 && bodySize / fullRange > 0.70;
  const isAtHigh = cur.close > cur.open;
  const l6 = isLargeBody && isAtHigh;

  return { l1, l2, l3, l4, l5, l6 };
}

function nullMatrix() {
  return { l1: false, l2: false, l3: false, l4: false, l5: false, l6: false };
}

// ── Structural Range ──────────────────────────────────────────────────────────

export function calculateStructuralRange(candles, lookback = 60) {
  if (candles.length < lookback) return { high: 0, low: 0, mean: 0 };
  const slice = candles.slice(-lookback);
  const high = Math.max(...slice.map(c => c.high));
  const low  = Math.min(...slice.map(c => c.low));
  const mean = slice.reduce((s, c) => s + c.close, 0) / lookback;
  return { high, low, mean };
}

// ── R-Score ───────────────────────────────────────────────────────────────────

const LAMBDA_WEIGHTS = { l1: 0.25, l2: 0.15, l3: 0.20, l4: 0.15, l5: 0.25 };

export function calculateRScore(matrix) {
  let score = 0;
  if (matrix.l1) score += LAMBDA_WEIGHTS.l1;
  if (matrix.l2) score += LAMBDA_WEIGHTS.l2;
  if (matrix.l3) score += LAMBDA_WEIGHTS.l3;
  if (matrix.l4) score += LAMBDA_WEIGHTS.l4;
  if (matrix.l5) score += LAMBDA_WEIGHTS.l5;
  return score;
}

// ── Mandra Gate ───────────────────────────────────────────────────────────────

export function mandraGate(candles) {
  if (candles.length < 2) return false;
  const cur  = candles[candles.length - 1];
  const prev = candles[candles.length - 2];
  const eCur  = cur.volume  * Math.abs(cur.close  - cur.open)  ** 2;
  const ePrev = prev.volume * Math.abs(prev.close - prev.open) ** 2;
  return eCur >= ePrev;
}

// ── System State ──────────────────────────────────────────────────────────────

export const SystemState = {
  DELIVERY:      'DELIVERY',
  REVERSE_PERIOD: 'REVERSE_PERIOD',
  CONSOLIDATION: 'CONSOLIDATION',
  HALTED:        'HALTED',
};

// ── Engine ────────────────────────────────────────────────────────────────────

const DEFAULT_CFG = {
  lookback20: 20,
  lookback40: 40,
  lookback60: 60,
  atrPeriod:  14,
  killzoneLondonStart: 8,
  killzoneLondonEnd:   11,
  killzoneNYStart:     13,
  killzoneNYEnd:       16,
};

export class ReversePeriodEngine {
  constructor(cfg = DEFAULT_CFG) {
    this.cfg      = cfg;
    this.state    = SystemState.DELIVERY;
    this.isDamped = false;
    this.candles  = this._generateMockHistory(120);
  }

  _generateMockHistory(count) {
    const candles = [];
    const now = Date.now();
    let price = 1.1000;
    for (let i = 0; i < count; i++) {
      const noise = (Math.random() - 0.5) * 0.0012;
      price += noise;
      const open  = price;
      const close = price + (Math.random() - 0.5) * 0.0003;
      const wick  = Math.random() * 0.0005;
      candles.push({
        timestamp: now - (count - i) * 3600000,
        open,
        high: Math.max(open, close) + wick,
        low:  Math.min(open, close) - wick,
        close,
        volume: 800 + Math.random() * 600,
      });
    }
    return candles;
  }

  tick(priceDelta) {
    const last = this.candles[this.candles.length - 1];
    const newClose = last.close + priceDelta;
    const noise   = (Math.random() - 0.5) * 0.00008;

    const newCandle = {
      timestamp: Date.now(),
      open:   last.close,
      high:   Math.max(last.close, newClose) + Math.abs(noise),
      low:    Math.min(last.close, newClose) - Math.abs(noise),
      close:  newClose,
      volume: 900 + Math.random() * 500,
    };
    this.candles.push(newCandle);

    // Keep history bounded
    if (this.candles.length > 500) this.candles.shift();

    // Compute volatility proxy
    const atr    = calculateATR(this.candles, 14) ?? 0.0001;
    const curVol = (newCandle.high - newCandle.low) / newCandle.close;

    // Position in range → confidence heuristic
    const rangeH = Math.max(...this.candles.slice(-60).map(c => c.high));
    const rangeL = Math.min(...this.candles.slice(-60).map(c => c.low));
    const pos    = (newCandle.close - rangeL) / (rangeH - rangeL || 1);
    const confidence = (pos > 0.88 || pos < 0.12) ? 0.82 : 0.28;

    // Run detection
    const matrix = analyzeDetectionMatrix(
      this.candles, this.cfg, curVol, -50, confidence
    );

    // Veto check
    if (matrix.l6) {
      this.state = SystemState.HALTED;
    } else {
      const rScore = calculateRScore(matrix);
      const lastC  = this.candles[this.candles.length - 1];
      const isCoherent    = adelicManifoldValidator(lastC.close, curVol, lastC.volume);
      const isDistribution = curVol < 0.001;
      const triggerFired   = isCoherent && isDistribution && rScore > 0.6;

      if (triggerFired) {
        // Mandra Gate check
        if (!mandraGate(this.candles)) {
          this.isDamped = true;
        } else {
          this._executeReversePeriod(matrix);
        }
      } else if (this.state === SystemState.REVERSE_PERIOD) {
        const range = calculateStructuralRange(this.candles);
        const dist  = Math.abs(lastC.close - range.mean);
        const span  = range.high - range.low;
        if (dist < 0.1 * span) this.state = SystemState.DELIVERY;
      } else if (this.state === SystemState.HALTED) {
        // Auto-recover after veto clears
        this.state = SystemState.DELIVERY;
      }
    }

    const range  = calculateStructuralRange(this.candles);
    const matrix2 = analyzeDetectionMatrix(
      this.candles, this.cfg, curVol, -50, confidence
    );
    const rScore = calculateRScore(matrix2);

    return {
      state:   this.state,
      rScore,
      matrix:  matrix2,
      range,
      target:  range.mean,
      isDamped: this.isDamped,
      atr,
      confidence,
    };
  }

  _executeReversePeriod(matrix) {
    const atr  = calculateATR(this.candles, 14) ?? 0.0001;
    const last = this.candles[this.candles.length - 1];
    const wickTop    = last.high - last.close;
    const wickBottom = last.close - last.low;
    const isSweep = (wickTop > 1.5 * atr || wickBottom > 1.5 * atr);

    if (!isSweep && !matrix.l3) return;

    // TDA loop check — last 5 bars ranging?
    const c5 = this.candles[this.candles.length - 5];
    const isLooping = c5 && Math.abs(last.close - c5.close) < atr;
    if (isLooping) return;

    this.state    = SystemState.REVERSE_PERIOD;
    this.isDamped = false;
  }

  getCandles() {
    return this.candles;
  }
}
