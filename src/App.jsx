import React, { useEffect, useRef, useState } from 'react';
import { BrowserRouter as Router, Routes, Route, Link, useLocation } from 'react-router-dom';
import { Activity, Search, Settings, AlertTriangle, Target, Zap, ChevronRight } from 'lucide-react';
import { createChart } from 'lightweight-charts';
import initWasm, { WasmEngine } from '../pkg/seven_zero_ipda'; // Adjust if path changed
import PatternScanner from './pages/PatternScanner';

// --- Components ---

const Layout = ({ children }) => {
  const location = useLocation();

  const navItems = [
    { path: '/', label: 'Live Monitor', icon: <Activity size={18} /> },
    { path: '/scanner', label: 'Pattern Scanner', icon: <Search size={18} /> },
    { path: '/settings', label: 'System Config', icon: <Settings size={18} /> },
  ];

  return (
    <div className="min-h-screen bg-slate-950 text-slate-200 flex flex-col font-sans">
      {/* Global Navigation */}
      <nav className="border-b border-slate-800 bg-slate-950/50 backdrop-blur-md sticky top-0 z-50">
        <div className="max-w-7xl mx-auto px-4 h-16 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="w-8 h-8 bg-cyan-500 rounded flex items-center justify-center text-slate-950">
              <Zap size={20} fill="currentColor" />
            </div>
            <span className="font-bold tracking-tighter text-lg text-white">7ZERO <span className="text-cyan-400">IPDA</span></span>
          </div>

          <div className="flex gap-1">
            {navItems.map((item) => (
              <Link
                key={item.path}
                to={item.path}
                className={`flex items-center gap-2 px-4 py-2 rounded-lg transition-all text-sm font-medium ${
                  location.pathname === item.path
                    ? 'bg-cyan-500/10 text-cyan-400 ring-1 ring-cyan-500/30'
                    : 'text-slate-400 hover:text-slate-100 hover:bg-slate-900'
                }`}
              >
                {item.icon}
                {item.label}
              </Link>
            ))}
          </div>
        </div>
      </nav>

      <main className="flex-1">
        {children}
      </main>
    </div>
  );
};

// Existing Live Monitor Logic Refactored into a Component
const LiveMonitor = () => {
  const [engine, setEngine] = useState(null);
  const [state, setState] = useState({
    status: 'INITIALIZING',
    rScore: 0,
    matrix: { l1: false, l2: false, l3: false, l4: false, l5: false, l6: false },
    range: { high: 0, low: 0, mean: 0 },
    target: 0
  });
  
  const chartContainerRef = useRef();
  const chartRef = useRef(null);
  const candleSeriesRef = useRef(null);
  const meanLineRef = useRef(null);

  useEffect(() => {
    const init = async () => {
      try {
        await initWasm();
        const wasmEngine = new WasmEngine();
        setEngine(wasmEngine);
        
        const chart = createChart(chartContainerRef.current, {
          width: chartContainerRef.current.clientWidth,
          height: 400,
          layout: { background: { color: '#020617' }, textColor: '#94a3b8' },
          grid: { vertLines: { color: '#0f172a' }, horzLines: { color: '#0f172a' } },
        });
        
        chartRef.current = chart;
        candleSeriesRef.current = chart.addCandlestickSeries({
          upColor: '#22c55e', downColor: '#ef4444', borderVisible: false,
          wickUpColor: '#22c55e', wickDownColor: '#ef4444',
        });

        const initialCandles = wasmEngine.get_candles();
        candleSeriesRef.current.setData(initialCandles.map(c => ({
          time: c.timestamp / 1000, open: c.open, high: c.high, low: c.low, close: c.close
        })));
      } catch (e) { console.error(e); }
    };
    init();
  }, []);

  useEffect(() => {
    if (!engine) return;
    const interval = setInterval(() => {
      const move = (Math.random() - 0.5) * 0.0020;
      const result = engine.tick(move);
      const parsed = JSON.parse(JSON.stringify(result));
      setState(parsed);

      const candles = engine.get_candles();
      const lastCandle = candles[candles.length - 1];
      candleSeriesRef.current.update({
        time: lastCandle.timestamp / 1000,
        open: lastCandle.open, high: lastCandle.high, low: lastCandle.low, close: lastCandle.close
      });
    }, 1000);
    return () => clearInterval(interval);
  }, [engine]);

  const LambdaIndicator = ({ label, active, color }) => (
    <div className={`flex items-center justify-between p-3 rounded-lg border transition-all ${active ? 'bg-slate-900 border-slate-700' : 'bg-transparent border-transparent opacity-30'}`}>
      <span className="text-xs font-mono text-slate-300">λ{label}</span>
      <div className={`w-2.5 h-2.5 rounded-full ${active ? 'animate-pulse' : ''}`} style={{ backgroundColor: active ? color : '#475569' }}></div>
    </div>
  );

  return (
    <div className="p-4 grid grid-cols-1 lg:grid-cols-4 gap-4 h-full">
      <div className="lg:col-span-3 flex flex-col gap-4">
        {/* Header Stats */}
        <div className="grid grid-cols-3 gap-4">
          <div className="bg-slate-900/50 p-4 rounded-xl border border-slate-800">
            <span className="text-[10px] font-bold text-slate-500 uppercase tracking-widest">System State</span>
            <div className="flex items-center gap-2 mt-1">
              <div className={`w-2 h-2 rounded-full ${state.state === 'REVERSE_PERIOD' ? 'bg-green-500 shadow-[0_0_10px_#22c55e]' : 'bg-cyan-500'}`} />
              <span className="text-lg font-bold text-white">{state.state}</span>
            </div>
          </div>
          <div className="bg-slate-900/50 p-4 rounded-xl border border-slate-800">
            <span className="text-[10px] font-bold text-slate-500 uppercase tracking-widest">Magnet / Mean</span>
            <div className="text-lg font-mono font-bold text-amber-400 mt-1">{state.range.mean.toFixed(5)}</div>
          </div>
          <div className="bg-slate-900/50 p-4 rounded-xl border border-slate-800">
            <span className="text-[10px] font-bold text-slate-500 uppercase tracking-widest">Unified R-Score</span>
            <div className="text-lg font-bold text-cyan-400 mt-1">{(state.rScore * 100).toFixed(0)}%</div>
          </div>
        </div>

        {/* Chart View */}
        <div className="bg-slate-900/50 rounded-2xl border border-slate-800 p-2 flex-1 relative overflow-hidden">
          <div className="absolute top-4 left-4 z-10 flex gap-2">
            <span className="px-2 py-1 bg-slate-950/80 rounded border border-slate-700 text-[10px] font-mono text-slate-400 uppercase">Real-Time Tick Feed</span>
          </div>
          <div ref={chartContainerRef} className="w-full h-full min-h-[500px]" />
        </div>
      </div>

      <div className="flex flex-col gap-4">
        {/* Lambda Matrix */}
        <div className="bg-slate-900/50 p-6 rounded-2xl border border-slate-800 flex flex-col gap-4">
          <div className="flex items-center justify-between">
            <h3 className="text-xs font-bold text-slate-500 uppercase tracking-widest flex items-center gap-2">
              <Zap size={14} className="text-cyan-400" /> Detection Matrix
            </h3>
            <span className="text-[10px] text-slate-600 font-mono">v0.1.0-α</span>
          </div>
          <div className="grid grid-cols-1 gap-2">
            <LambdaIndicator label="1 - Entrapment" active={state.matrix.l1} color="#3b82f6" />
            <LambdaIndicator label="2 - Temporal" active={state.matrix.l2} color="#8b5cf6" />
            <LambdaIndicator label="3 - Spectral" active={state.matrix.l3} color="#ec4899" />
            <LambdaIndicator label="4 - Confluence" active={state.matrix.l4} color="#14b8a6" />
            <LambdaIndicator label="5 - Liquidity" active={state.matrix.l5} color="#f97316" />
            <LambdaIndicator label="6 - VETO" active={state.matrix.l6} color="#ef4444" />
          </div>

          {state.matrix.l6 && (
            <div className="p-3 bg-red-900/20 border border-red-500/30 rounded-lg text-red-400 text-xs flex items-center gap-3">
              <AlertTriangle size={16} />
              <span className="leading-tight">DISPLACEMENT VETO: Structural conflict detected. System Halted.</span>
            </div>
          )}
        </div>

        {/* Structural Info */}
        <div className="bg-slate-900/50 p-6 rounded-2xl border border-slate-800">
           <h3 className="text-xs font-bold text-slate-500 uppercase tracking-widest flex items-center gap-2 mb-4">
              <Target size={14} className="text-cyan-400" /> Structural Box
            </h3>
            <div className="space-y-4">
              <div>
                <div className="flex justify-between text-[10px] text-slate-500 font-bold uppercase mb-1">
                  <span>Price Range (L60)</span>
                  <span className="text-slate-400 font-mono">{(state.range.high - state.range.low).toFixed(5)} pips</span>
                </div>
                <div className="h-2 bg-slate-800 rounded-full overflow-hidden flex">
                  <div className="bg-slate-700 h-full border-r border-slate-600" style={{ width: '33%' }}></div>
                  <div className="bg-slate-700 h-full border-r border-slate-600" style={{ width: '33%' }}></div>
                </div>
              </div>

              <div className="grid grid-cols-2 gap-4 font-mono text-sm">
                <div className="p-2 bg-slate-950/50 rounded border border-slate-800">
                  <div className="text-[10px] text-slate-500">HIGH</div>
                  <div className="text-red-400">{state.range.high.toFixed(5)}</div>
                </div>
                <div className="p-2 bg-slate-950/50 rounded border border-slate-800">
                  <div className="text-[10px] text-slate-500">LOW</div>
                  <div className="text-green-400">{state.range.low.toFixed(5)}</div>
                </div>
              </div>
            </div>
        </div>
      </div>
    </div>
  );
};

function App() {
  return (
    <Router>
      <Layout>
        <Routes>
          <Route path="/" element={<LiveMonitor />} />
          <Route path="/scanner" element={<PatternScanner />} />
          <Route path="/settings" element={<div className="p-10 text-slate-500">Settings Module: Threshold and λ-Weight Configuration Coming Soon.</div>} />
        </Routes>
      </Layout>
    </Router>
  );
}

export default App;
