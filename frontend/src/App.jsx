import React, { useEffect, useRef, useState, useCallback } from 'react';
import { createChart, CrosshairMode, CandlestickSeries, LineSeries } from 'lightweight-charts';
import { Activity, AlertTriangle, Target, Zap, Radio, Cpu, TrendingUp, Shield } from 'lucide-react';
import { ReversePeriodEngine as JsEngine, SystemState } from './engine.js';
import initWasm, { WasmEngine } from './pkg/seven_zero_ipda.js';
import wasmUrl from './pkg/seven_zero_ipda_bg.wasm?url';

// ── Helpers ───────────────────────────────────────────────────────────────────

function useEngineState() {
  const [engine, setEngine] = useState(null);
  const [useWasm, setUseWasm] = useState(false);
  const [wasmReady, setWasmReady] = useState(false);

  useEffect(() => {
    const loadWasm = async () => {
      try {
        await initWasm(wasmUrl);
        setWasmReady(true);
      } catch (e) {
        console.error("WASM Load Error", e);
      }
    };
    loadWasm();
  }, []);

  useEffect(() => {
    if (useWasm && wasmReady) {
      setEngine(new WasmEngine());
    } else {
      setEngine(new JsEngine());
    }
  }, [useWasm, wasmReady]);

  return { engine, useWasm, setUseWasm, wasmReady };
}

function formatPrice(p) {
  return typeof p === 'number' && isFinite(p) ? p.toFixed(5) : '-.-----';
}

function formatPct(p) {
  return typeof p === 'number' && isFinite(p) ? (p * 100).toFixed(1) + '%' : '-.-%';
}

function useClock() {
  const [now, setNow] = useState(new Date());
  useEffect(() => {
    const id = setInterval(() => setNow(new Date()), 1000);
    return () => clearInterval(id);
  }, []);
  return now.toUTCString().slice(17, 25) + ' UTC';
}

function stateClass(s) {
  if (s === SystemState.REVERSE_PERIOD) return 'reverse';
  if (s === SystemState.HALTED)         return 'halted';
  if (s === SystemState.CONSOLIDATION)  return 'consolidation';
  return 'delivery';
}

function rscoreClass(r) {
  if (r >= 0.6) return 'high';
  if (r >= 0.35) return 'mid';
  return 'low';
}

// ── Lambda Indicator ──────────────────────────────────────────────────────────

const LAMBDA_META = [
  { key: 'l1', sym: 'λ1', name: 'PHASE ENTRAP.' },
  { key: 'l2', sym: 'λ2', name: 'TEMPORAL ALN.' },
  { key: 'l3', sym: 'λ3', name: 'SPECTRAL INV.' },
  { key: 'l4', sym: 'λ4', name: 'CONFLUENC COL.' },
  { key: 'l5', sym: 'λ5', name: 'LIQUIDITY INV.' },
  { key: 'l6', sym: 'λ6', name: 'DISP. VETO' },
];

function LambdaItem({ meta, active }) {
  return (
    <div
      className={`lambda-item ${active ? 'active' : 'inactive'}`}
      data-lambda={meta.key}
      title={meta.name}
    >
      <div className="lambda-item-left">
        <span className="lambda-key">{meta.sym}</span>
        <span className="lambda-name">{meta.name}</span>
      </div>
      <div className="lambda-dot" />
    </div>
  );
}

// ── Main App ──────────────────────────────────────────────────────────────────

export default function App() {
  const { engine, useWasm, setUseWasm, wasmReady } = useEngineState();
  const clock  = useClock();

  const [output, setOutput] = useState({
    state:    SystemState.DELIVERY,
    rScore:   0,
    matrix:   { l1: false, l2: false, l3: false, l4: false, l5: false, l6: false },
    range:    { high: 1.1005, low: 1.0995, mean: 1.1000 },
    target:   1.1000,
    isDamped: false,
    atr:      0.0005,
    confidence: 0.3,
  });

  const [tickCount,   setTickCount]   = useState(0);
  const [triggerLog,  setTriggerLog]  = useState([]);
  const [lastPrice,   setLastPrice]   = useState(1.1000);
  const [priceChange, setPriceChange] = useState(0);

  // Chart refs
  const chartContainerRef = useRef(null);
  const chartRef          = useRef(null);
  const candleSeriesRef   = useRef(null);
  const meanLineRef       = useRef(null);
  const highLineRef       = useRef(null);
  const lowLineRef        = useRef(null);

  // ── Chart Init ─────────────────────────────────────────────────────────────
  useEffect(() => {
    if (!chartContainerRef.current || !engine) return;

    const chart = createChart(chartContainerRef.current, {
      width:  chartContainerRef.current.clientWidth,
      height: chartContainerRef.current.clientHeight || 420,
      layout: {
        background: { color: '#0d1e35' },
        textColor:  '#475569',
      },
      grid: {
        vertLines: { color: '#0a1628', style: 1 },
        horzLines: { color: '#0a1628', style: 1 },
      },
      crosshair: {
        mode: CrosshairMode.Normal,
        vertLine: { color: '#1e3a5f', labelBackgroundColor: '#0d1e35' },
        horzLine: { color: '#1e3a5f', labelBackgroundColor: '#0d1e35' },
      },
      rightPriceScale: {
        borderColor: '#1a2f4a',
        textColor:   '#475569',
      },
      timeScale: {
        borderColor:     '#1a2f4a',
        textColor:       '#475569',
        timeVisible:     true,
        secondsVisible:  false,
      },
      handleScroll:  true,
      handleScale:   true,
    });

    chartRef.current = chart;

    // Candlestick series
    candleSeriesRef.current = chart.addSeries(CandlestickSeries, {
      upColor:       '#00ff9d',
      downColor:     '#ff4060',
      borderVisible: false,
      wickUpColor:   '#00cc7d',
      wickDownColor: '#cc3350',
    });

    // Mean line
    meanLineRef.current = chart.addSeries(LineSeries, {
      color:     '#f59e0b',
      lineWidth: 1,
      lineStyle: 2, // dashed
      title:     'EQ',
      priceLineVisible: false,
      lastValueVisible: true,
    });

    // High line
    highLineRef.current = chart.addSeries(LineSeries, {
      color:     'rgba(255, 64, 96, 0.5)',
      lineWidth: 1,
      lineStyle: 1,
      title:     'H60',
      priceLineVisible: false,
      lastValueVisible: false,
    });

    // Low line
    lowLineRef.current = chart.addSeries(LineSeries, {
      color:     'rgba(0, 255, 157, 0.5)',
      lineWidth: 1,
      lineStyle: 1,
      title:     'L60',
      priceLineVisible: false,
      lastValueVisible: false,
    });

    // Seed initial candles
    const candles = engine.get_candles ? engine.get_candles() : engine.getCandles();
    const chartData = candles.map(c => ({
      time:  Math.floor(c.timestamp / 1000),
      open:  c.open,
      high:  c.high,
      low:   c.low,
      close: c.close,
    }));
    candleSeriesRef.current.setData(chartData);

    // Resize observer
    const ro = new ResizeObserver(() => {
      if (chartContainerRef.current) {
        chart.applyOptions({ width: chartContainerRef.current.clientWidth });
      }
    });
    ro.observe(chartContainerRef.current);

    return () => {
      ro.disconnect();
      chart.remove();
    };
  }, [engine]);

  // ── Simulation Loop ────────────────────────────────────────────────────────
  useEffect(() => {
    if (!engine) return;

    const TICK_MS = 800;

    const id = setInterval(() => {
      // Simulate market — slightly biased random walk with mean reversion
      const candles = engine.get_candles ? engine.get_candles() : engine.getCandles();
      if (!candles || candles.length === 0) return;

      const last    = candles[candles.length - 1];
      const range   = output.range;
      const span    = range.high - range.low || 0.001;
      const pos     = (last.close - range.low) / span;

      // Mean reversion bias
      const bias  = (0.5 - pos) * 0.0003;
      const noise = (Math.random() - 0.5) * 0.0018;
      const delta = bias + noise;

      const result = engine.tick(delta);
      
      // The WASM engine already returns camelCase due to its Serde configuration.
      // We only need to ensure consistent object structure if needed, but result should be fine.
      const normalizedResult = result;

      setOutput(normalizedResult);

      const newCandles = engine.get_candles ? engine.get_candles() : engine.getCandles();
      const lc = newCandles[newCandles.length - 1];
      const prevClose = newCandles[newCandles.length - 2]?.close ?? lc.close;

      setLastPrice(lc.close);
      setPriceChange(lc.close - prevClose);
      setTickCount(t => t + 1);

      // Update chart candle
      if (candleSeriesRef.current) {
        candleSeriesRef.current.update({
          time:  Math.floor(lc.timestamp / 1000),
          open:  lc.open,
          high:  lc.high,
          low:   lc.low,
          close: lc.close,
        });
      }

      // Update structural lines
      const t = Math.floor(lc.timestamp / 1000);
      const tStart = t - 7200; // 2h back

      if (meanLineRef.current && result.range.mean > 0) {
        meanLineRef.current.setData([
          { time: tStart, value: result.range.mean },
          { time: t,      value: result.range.mean },
        ]);
      }
      if (highLineRef.current && result.range.high > 0) {
        highLineRef.current.setData([
          { time: tStart, value: result.range.high },
          { time: t,      value: result.range.high },
        ]);
      }
      if (lowLineRef.current && result.range.low > 0) {
        lowLineRef.current.setData([
          { time: tStart, value: result.range.low },
          { time: t,      value: result.range.low },
        ]);
      }

      // Log state transitions
      if (result.state === SystemState.REVERSE_PERIOD) {
        setTriggerLog(log => {
          const entry = {
            ts: new Date().toISOString().slice(11, 19),
            price: lc.close.toFixed(5),
            rScore: (result.rScore * 100).toFixed(0),
          };
          const updated = [entry, ...log].slice(0, 5);
          return updated;
        });
      }
    }, TICK_MS);

    return () => clearInterval(id);
  }, [engine]);

  // ── Derived UI ─────────────────────────────────────────────────────────────
  const sc         = stateClass(output.state);
  const rc         = rscoreClass(output.rScore);
  const activeCount = Object.values(output.matrix).filter(Boolean).length;
  const spread     = output.range.high - output.range.low;

  return (
    <div className="terminal-root">
      {/* ── Header ── */}
      <header className="header">
        <div className="header-brand">
          <div className="header-logo">
            <svg viewBox="0 0 32 32" fill="none">
              <polygon points="16,2 30,26 2,26" fill="none" stroke="#00d4ff" strokeWidth="1.5" />
              <polygon points="16,9 25,23 7,23" fill="rgba(0,212,255,0.08)" stroke="#00d4ff" strokeWidth="1" opacity="0.6" />
              <circle cx="16" cy="16" r="2" fill="#00d4ff" />
            </svg>
          </div>
          <div>
            <div className="header-title">7ZERO</div>
            <div className="header-subtitle">IPDA Reverse Period Terminal</div>
          </div>
        </div>

        <div className="header-right">
          <div className="header-clock">{clock}</div>
          
          {/* Data Source Selector */}
          <select 
            className="status-badge delivery" 
            style={{ 
              background: 'var(--bg-deep)', 
              color: 'var(--text-bright)', 
              border: '1px solid var(--border)',
              padding: '2px 8px',
              marginRight: 8,
              fontSize: 10,
              fontFamily: 'var(--font-mono)',
              borderRadius: 4,
              cursor: 'pointer'
            }}
            onChange={async (e) => {
              const mode = e.target.value;
              if (engine && engine.load_data) {
                try {
                  console.log(`Switching to ${mode} mode...`);
                  await engine.load_data(mode);
                  // Refresh chart data
                  const candles = engine.get_candles ? engine.get_candles() : engine.getCandles();
                  const chartData = candles.map(c => ({
                    time:  Math.floor(c.timestamp / 1000),
                    open:  c.open,
                    high:  c.high,
                    low:   c.low,
                    close: c.close,
                  }));
                  candleSeriesRef.current.setData(chartData);
                } catch (err) {
                  alert(`Failed to load ${mode} data: ${err}`);
                }
              }
            }}
          >
            <option value="MOCK">SIMULATION</option>
            <option value="BITGET">BITGET LIVE</option>
            <option value="XM">XM FOREX</option>
          </select>

          <button 
            className={`status-badge ${useWasm ? 'reverse' : 'delivery'}`}
            onClick={() => wasmReady && setUseWasm(!useWasm)}
            disabled={!wasmReady}
            style={{ cursor: 'pointer', outline: 'none' }}
          >
            <Cpu size={10} style={{ marginRight: 4 }} />
            {useWasm ? 'WASM CORE' : 'JS SIM'}
          </button>

          <div className={`status-badge ${sc}`}>
            <div className="status-dot" />
            {output.state.replace('_', ' ')}
          </div>
        </div>
      </header>

      {/* ── Main Grid ── */}
      <main className="main-grid">

        {/* ── Chart Panel ── */}
        <div className="panel chart-panel">
          <div className="panel-header">
            <TrendingUp size={12} color="var(--cyan)" />
            <span className="panel-label panel-label-accent">EURUSD</span>
            <span className="panel-label" style={{ marginLeft: 4 }}>· H1 · SIM</span>
            <span
              className="panel-label"
              style={{
                marginLeft: 12,
                color: priceChange >= 0 ? 'var(--green)' : 'var(--red)',
                fontWeight: 700,
                fontSize: 12,
              }}
            >
              {formatPrice(lastPrice)}
              <span style={{ fontSize: 9, marginLeft: 6, opacity: 0.8 }}>
                {priceChange >= 0 ? '▲' : '▼'} {formatPrice(Math.abs(priceChange))}
              </span>
            </span>
            <span className="panel-tag">L60 STRUCTURAL BOX</span>
          </div>
          <div className="chart-container" ref={chartContainerRef}>
            <div className="chart-overlay-info">
              <div className="chart-overlay-chip">
                ATR<span>{output.atr?.toFixed(5) ?? '-.-----'}</span>
              </div>
              <div className="chart-overlay-chip">
                TICK<span>#{tickCount}</span>
              </div>
              <div className="chart-overlay-chip">
                CONF<span>{formatPct(output.confidence)}</span>
              </div>
            </div>
          </div>
        </div>

        {/* ── Sidebar ── */}
        <div className="sidebar">

          {/* Structural Box */}
          <div className="panel box-panel">
            <div className="panel-header">
              <Target size={12} color="var(--cyan)" />
              <span className="panel-label panel-label-accent">STRUCTURAL BOX</span>
              <span className="panel-tag">L60</span>
            </div>
            <div className="panel-body">
              <div className="box-row high">
                <span className="box-row-label">PREMIUM (H)</span>
                <span className="box-row-value">{formatPrice(output.range.high)}</span>
              </div>
              <div className="box-divider" />
              <div className="box-row mean">
                <span className="box-row-label">EQUILIBRIUM ⟵</span>
                <span className="box-row-value">{formatPrice(output.range.mean)}</span>
              </div>
              <div className="box-divider" />
              <div className="box-row low">
                <span className="box-row-label">DISCOUNT (L)</span>
                <span className="box-row-value">{formatPrice(output.range.low)}</span>
              </div>
              <div className="box-spread">
                <span className="box-spread-label">SPREAD</span>
                <span className="box-spread-value">{spread > 0 ? (spread * 10000).toFixed(1) + ' pips' : '—'}</span>
              </div>
              <div className="mandra-row">
                <span className="mandra-label">MANDRA GATE</span>
                <span className={`mandra-status ${output.isDamped ? 'damped' : 'ok'}`}>
                  {output.isDamped ? 'DAMPED' : 'CLEAR'}
                </span>
              </div>
            </div>
          </div>

          {/* Lambda Detection Matrix */}
          <div className="panel matrix-panel">
            <div className="panel-header">
              <Zap size={12} color="var(--cyan)" />
              <span className="panel-label panel-label-accent">DETECTION MATRIX</span>
              <span className="panel-tag">{activeCount}/6 ACTIVE</span>
            </div>
            <div className="panel-body">
              <div className="lambda-grid">
                {LAMBDA_META.map(m => (
                  <LambdaItem key={m.key} meta={m} active={!!output.matrix[m.key]} />
                ))}
              </div>
              {output.matrix.l6 && (
                <div className="veto-alert">
                  <span className="veto-alert-icon">⚠</span>
                  <span className="veto-alert-text">DISPLACEMENT VETO ACTIVE — EXECUTION HALTED</span>
                </div>
              )}
            </div>
          </div>

          {/* R-Score */}
          <div className="panel rscore-panel">
            <div className="panel-header">
              <Activity size={12} color="var(--cyan)" />
              <span className="panel-label panel-label-accent">R-SCORE</span>
              <span className="panel-tag">θ = 60%</span>
            </div>
            <div className="panel-body">
              <div className="rscore-header">
                <span className="rscore-label">REVERSE PERIOD SCORE</span>
                <span className={`rscore-value ${rc}`}>{(output.rScore * 100).toFixed(0)}%</span>
              </div>
              <div className="rscore-bar-track">
                <div
                  className={`rscore-bar-fill ${rc}`}
                  style={{ width: `${Math.min(output.rScore * 100, 100)}%` }}
                />
              </div>
              <div className="rscore-threshold">
                <span className="rscore-threshold-label">THRESHOLD</span>
                <span className="rscore-threshold-value">60%</span>
              </div>

              {/* Trigger log */}
              {triggerLog.length > 0 && (
                <div style={{ marginTop: 12 }}>
                  <div className="panel-label" style={{ marginBottom: 6 }}>TRIGGER LOG</div>
                  {triggerLog.map((e, i) => (
                    <div key={i} style={{
                      display: 'flex', justifyContent: 'space-between',
                      padding: '4px 6px', marginBottom: 2,
                      background: 'var(--bg-deep)',
                      borderRadius: 3,
                      borderLeft: '2px solid var(--green)',
                    }}>
                      <span style={{ fontFamily: 'var(--font-mono)', fontSize: 8, color: 'var(--text-dim)' }}>{e.ts}</span>
                      <span style={{ fontFamily: 'var(--font-mono)', fontSize: 8, color: 'var(--green)' }}>{e.price}</span>
                      <span style={{ fontFamily: 'var(--font-mono)', fontSize: 8, color: 'var(--amber)' }}>{e.rScore}%</span>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </div>
        </div>

        {/* ── Metrics Row ── */}
        <div className="metrics-row" style={{ gridColumn: '1 / -1' }}>
          <div className="metric-card c-cyan" style={{ animationDelay: '0ms' }}>
            <div className="metric-label">
              <Radio size={8} style={{ display:'inline', marginRight:4 }} />
              SYSTEM STATE
            </div>
            <div className="metric-value" style={{ fontSize: 14 }}>{output.state.replace('_', ' ')}</div>
            <div className="metric-sublabel">IPDA ENGINE v0.1</div>
          </div>

          <div className="metric-card c-green" style={{ animationDelay: '60ms' }}>
            <div className="metric-label">
              <TrendingUp size={8} style={{ display:'inline', marginRight:4 }} />
              ACTIVE SIGNALS
            </div>
            <div className="metric-value">{activeCount} / 6</div>
            <div className="metric-sublabel">LAMBDA FILTERS</div>
          </div>

          <div className="metric-card c-amber" style={{ animationDelay: '120ms' }}>
            <div className="metric-label">
              <Target size={8} style={{ display:'inline', marginRight:4 }} />
              TARGET PRICE
            </div>
            <div className="metric-value" style={{ fontSize: 15 }}>{formatPrice(output.target)}</div>
            <div className="metric-sublabel">EQ / L60 MEAN</div>
          </div>

          <div className="metric-card c-purple" style={{ animationDelay: '180ms' }}>
            <div className="metric-label">
              <Shield size={8} style={{ display:'inline', marginRight:4 }} />
              SIGNAL CONF.
            </div>
            <div className="metric-value">{formatPct(output.confidence)}</div>
            <div className="metric-sublabel">ADELIC COHERENCE</div>
          </div>
        </div>

      </main>

      {/* ── Footer ── */}
      <footer className="footer">
        <span className="footer-info">
          7ZERO-REVERSE · <span>IPDA ENGINE</span> · Rust/WASM Core · Simulation Mode
        </span>
        <div className="footer-signal">
          <div className="signal-bar">
            <div /><div /><div /><div />
          </div>
          LIVE · {clock}
        </div>
        <span className="footer-info">
          λ1…λ6 · ADELIC MANIFOLD VALIDATOR · MANDRA GATE · <span>MIT LICENSE</span>
        </span>
      </footer>
    </div>
  );
}
