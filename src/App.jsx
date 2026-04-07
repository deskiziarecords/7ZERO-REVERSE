import React, { useEffect, useRef, useState } from 'react';
import { createChart, IChartApi, ISeriesApi, CandlestickData, LineData } from 'lightweight-charts';
import { Activity, AlertTriangle, Target, Zap } from 'lucide-react';
import initWasm, { WasmEngine } from '../pkg/seven_zero_ipda'; // Import generated WASM

const SevenZeroDashboard = () => {
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
  const rangeLinesRef = useRef([]);

  // Initialize WASM and Chart
  useEffect(() => {
    const init = async () => {
      try {
        await initWasm();
        const wasmEngine = new WasmEngine();
        setEngine(wasmEngine);
        
        // Setup Chart
        const chart = createChart(chartContainerRef.current, {
          width: chartContainerRef.current.clientWidth,
          height: 400,
          layout: {
            background: { color: '#0f172a' }, // Slate 900
            textColor: '#94a3b8',
          },
          grid: {
            vertLines: { color: '#1e293b' },
            horzLines: { color: '#1e293b' },
          },
        });
        
        chartRef.current = chart;
        candleSeriesRef.current = chart.addCandlestickSeries({
          upColor: '#10b981', // Emerald 500
          downColor: '#ef4444', // Red 500
          borderVisible: false,
          wickUpColor: '#10b981',
          wickDownColor: '#ef4444',
        });

        // Load initial data
        const initialCandles = wasmEngine.get_candles();
        const data = initialCandles.map(c => ({
          time: c.timestamp / 1000, // Convert ms to seconds
          open: c.open,
          high: c.high,
          low: c.low,
          close: c.close
        }));
        candleSeriesRef.current.setData(data);

        // Resize handler
        window.addEventListener('resize', () => {
          chart.applyOptions({ 
            width: chartContainerRef.current.clientWidth 
          });
        });
      } catch (e) {
        console.error("Failed to load WASM", e);
      }
    };

    init();
  }, []);

  // Simulation Loop
  useEffect(() => {
    if (!engine) return;

    const interval = setInterval(() => {
      // Simulate Market Movement (Random Walk)
      const move = (Math.random() - 0.5) * 0.0020;
      
      // Update Engine Logic
      const result = engine.tick(move);
      const parsed = JSON.parse(JSON.stringify(result)); // Clean serde proxy
      
      setState(parsed);

      // Update Chart
      const candles = engine.get_candles();
      const lastCandle = candles[candles.length - 1];
      
      candleSeriesRef.current.update({
        time: lastCandle.timestamp / 1000,
        open: lastCandle.open,
        high: lastCandle.high,
        low: lastCandle.low,
        close: lastCandle.close
      });

      // Update Structural Range Lines (Mean, High, Low)
      // Note: In a real app, manage series lifecycle properly
      if (meanLineRef.current) chartRef.current.removeSeries(meanLineRef.current);
      
      meanLineRef.current = chartRef.current.addLineSeries({ 
        color: '#f59e0b', // Amber
        lineWidth: 2,
        title: 'Equilibrium'
      });
      
      // Draw Mean Line
      const time = lastCandle.timestamp / 1000;
      meanLineRef.current.setData([
        { time: time - 1000, value: parsed.range.mean },
        { time: time + 100, value: parsed.range.mean }
      ]);

    }, 1000); // 1 second tick

    return () => clearInterval(interval);
  }, [engine]);

  // UI Helpers
  const LambdaIndicator = ({ label, active, color }) => (
    <div className={`flex items-center justify-between p-2 rounded ${active ? 'bg-opacity-20' : 'opacity-40'} transition-all`} style={{ backgroundColor: active ? color + '33' : 'transparent' }}>
      <span className="text-xs font-mono text-slate-300">λ{label}</span>
      <div className={`w-2 h-2 rounded-full ${active ? 'animate-pulse' : ''}`} style={{ backgroundColor: active ? color : '#475569' }}></div>
    </div>
  );

  return (
    <div className="min-h-screen bg-slate-950 text-slate-200 font-sans p-4 flex flex-col gap-4">
      {/* Header */}
      <header className="flex justify-between items-center border-b border-slate-800 pb-4">
        <div className="flex items-center gap-2">
          <Activity className="text-cyan-400" />
          <h1 className="text-xl font-bold tracking-wider text-cyan-400">7 ZERO <span className="text-slate-500 text-sm font-normal">REVERSE PERIOD TERMINAL</span></h1>
        </div>
        <div className={`px-4 py-1 rounded-full text-sm font-bold border ${
          state.status === 'REVERSE_PERIOD' ? 'border-green-500 text-green-400 bg-green-900/20' :
          state.status === 'HALTED' ? 'border-red-500 text-red-400 bg-red-900/20' :
          'border-slate-700 text-slate-400'
        }`}>
          STATUS: {state.status}
        </div>
      </header>

      <div className="grid grid-cols-1 lg:grid-cols-4 gap-4 flex-1">
        {/* Chart Section */}
        <div className="lg:col-span-3 bg-slate-900 rounded-lg border border-slate-800 p-1 flex flex-col">
          <div ref={chartContainerRef} className="w-full h-full min-h-[400px]" />
        </div>

        {/* Controls & Metrics */}
        <div className="flex flex-col gap-4">
          
          {/* Structural Box Info */}
          <div className="bg-slate-900 p-4 rounded-lg border border-slate-800">
            <h3 className="text-xs font-bold text-slate-500 uppercase mb-3 flex items-center gap-2">
              <Target size={14} /> Structural Box (L60)
            </h3>
            <div className="space-y-2 font-mono text-sm">
              <div className="flex justify-between">
                <span className="text-slate-400">High:</span>
                <span className="text-red-400">{state.range.high.toFixed(5)}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-slate-400">Mean (Magnet):</span>
                <span className="text-amber-400 font-bold">{state.range.mean.toFixed(5)}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-slate-400">Low:</span>
                <span className="text-green-400">{state.range.low.toFixed(5)}</span>
              </div>
            </div>
          </div>

          {/* Lambda Matrix */}
          <div className="bg-slate-900 p-4 rounded-lg border border-slate-800 flex-1">
            <h3 className="text-xs font-bold text-slate-500 uppercase mb-3 flex items-center gap-2">
              <Zap size={14} /> Detection Matrix
            </h3>
            <div className="grid grid-cols-2 gap-2">
              <LambdaIndicator label="1 (Entrapment)" active={state.matrix.l1} color="#3b82f6" />
              <LambdaIndicator label="2 (Temporal)" active={state.matrix.l2} color="#8b5cf6" />
              <LambdaIndicator label="3 (Spectral)" active={state.matrix.l3} color="#ec4899" />
              <LambdaIndicator label="4 (Confluence)" active={state.matrix.l4} color="#14b8a6" />
              <LambdaIndicator label="5 (Liquidity)" active={state.matrix.l5} color="#f97316" />
              <LambdaIndicator label="6 (VETO)" active={state.matrix.l6} color="#ef4444" />
            </div>
            
            {state.matrix.l6 && (
              <div className="mt-4 p-2 bg-red-900/30 border border-red-500/50 rounded text-red-300 text-xs flex items-center gap-2">
                <AlertTriangle size={12} /> DISPLACEMENT VETO ACTIVE
              </div>
            )}
          </div>

          {/* Severity Score */}
          <div className="bg-slate-900 p-4 rounded-lg border border-slate-800">
            <div className="flex justify-between items-end mb-1">
              <span className="text-xs font-bold text-slate-500">R-SCORE</span>
              <span className="text-2xl font-bold text-cyan-400">{(state.rScore * 100).toFixed(0)}%</span>
            </div>
            <div className="w-full bg-slate-800 h-2 rounded-full overflow-hidden">
              <div 
                className={`h-full transition-all duration-500 ${state.rScore > 0.6 ? 'bg-red-500' : 'bg-cyan-500'}`} 
                style={{ width: `${state.rScore * 100}%` }}
              ></div>
            </div>
            <p className="text-[10px] text-slate-500 mt-1 text-right">THRESHOLD: 60%</p>
          </div>
        </div>
      </div>
    </div>
  );
};

export default SevenZeroDashboard;
