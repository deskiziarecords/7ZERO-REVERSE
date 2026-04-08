import React, { useState } from 'react';
import { Search, TrendingUp, TrendingDown, Clock } from 'lucide-react';

const PatternScanner = () => {
  const [sequence, setSequence] = useState('');
  const [results, setResults] = useState(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);

  const handleScan = async () => {
    if (!sequence) return;
    setLoading(true);
    setError(null);
    try {
      const response = await fetch('http://localhost:8000/api/scan_sequence', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ sequence: sequence.toUpperCase(), lookback: 2000 })
      });
      if (!response.ok) throw new Error('Scanner API failed');
      const data = await response.json();
      setResults(data);
    } catch (err) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="p-6 bg-slate-950 min-h-screen text-slate-200">
      <div className="max-w-4xl mx-auto space-y-6">
        <header>
          <h1 className="text-2xl font-bold text-cyan-400 flex items-center gap-2">
            <Search size={24} /> PATTERN SCANNER / BACKTESTER
          </h1>
          <p className="text-slate-500 text-sm">Search historical patterns to find probabilistic edges.</p>
        </header>

        {/* Search Bar */}
        <div className="flex gap-4">
          <input
            type="text"
            value={sequence}
            onChange={(e) => setSequence(e.target.value)}
            placeholder="Enter sequence (e.g., IXWXB)"
            className="flex-1 bg-slate-900 border border-slate-800 p-3 rounded-lg font-mono text-cyan-400 focus:outline-none focus:border-cyan-500"
          />
          <button
            onClick={handleScan}
            disabled={loading}
            className="bg-cyan-600 hover:bg-cyan-500 disabled:bg-slate-700 px-6 py-3 rounded-lg font-bold transition-colors"
          >
            {loading ? 'SCANNING...' : 'START SCAN'}
          </button>
        </div>

        {error && <div className="p-4 bg-red-900/20 border border-red-500 text-red-400 rounded-lg">{error}</div>}

        {results && (
          <div className="space-y-6 animate-in fade-in slide-in-from-bottom-4 duration-500">
            {/* Stats Grid */}
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
              <div className="bg-slate-900 p-4 rounded-lg border border-slate-800 text-center">
                <div className="text-slate-500 text-xs uppercase font-bold mb-1">Matches Found</div>
                <div className="text-3xl font-bold text-white">{results.matches_found}</div>
              </div>
              <div className="bg-slate-900 p-4 rounded-lg border border-slate-800 text-center">
                <div className="text-slate-500 text-xs uppercase font-bold mb-1">Avg PnL (5 Bars)</div>
                <div className={`text-3xl font-bold ${results.avg_pnl >= 0 ? 'text-green-400' : 'text-red-400'}`}>
                  {results.avg_pnl} pips
                </div>
              </div>
              <div className="bg-slate-900 p-4 rounded-lg border border-slate-800 text-center">
                <div className="text-slate-500 text-xs uppercase font-bold mb-1">Win Rate</div>
                <div className="text-3xl font-bold text-cyan-400">
                  {results.results.length > 0
                    ? ((results.results.filter(r => r.outcome_5bars > 0).length / results.results.length) * 100).toFixed(0)
                    : 0}%
                </div>
              </div>
            </div>

            {/* Detailed Table */}
            <div className="bg-slate-900 rounded-lg border border-slate-800 overflow-hidden">
              <table className="w-full text-left border-collapse">
                <thead className="bg-slate-800 text-slate-400 text-xs uppercase">
                  <tr>
                    <th className="p-4"><Clock size={14} className="inline mr-1" /> Time Found</th>
                    <th className="p-4"><TrendingUp size={14} className="inline mr-1" /> Outcome (5b)</th>
                    <th className="p-4">High Reached</th>
                    <th className="p-4">Low Reached</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-slate-800">
                  {results.results.map((match, i) => (
                    <tr key={i} className="hover:bg-slate-800/50 transition-colors">
                      <td className="p-4 font-mono text-sm">
                        {new Date(match.found_at * 1000).toLocaleString()}
                      </td>
                      <td className={`p-4 font-bold ${match.outcome_5bars >= 0 ? 'text-green-400' : 'text-red-400'}`}>
                        {match.outcome_5bars > 0 ? '+' : ''}{match.outcome_5bars}
                      </td>
                      <td className="p-4 text-slate-400">{match.high_reached.toFixed(5)}</td>
                      <td className="p-4 text-slate-400">{match.low_reached.toFixed(5)}</td>
                    </tr>
                  ))}
                  {results.results.length === 0 && (
                    <tr>
                      <td colSpan="4" className="p-10 text-center text-slate-600">No recent occurrences found</td>
                    </tr>
                  )}
                </tbody>
              </table>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

export default PatternScanner;
