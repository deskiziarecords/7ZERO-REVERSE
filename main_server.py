from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
import pandas as pd
import random
from datetime import datetime

app = FastAPI()

# Enable CORS for React frontend
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_methods=["*"],
    allow_headers=["*"],
)

class PatternStreamer:
    def __init__(self):
        self.full_history = []

    def add_candle(self, candle):
        # In a real app, this would be called by the Rust engine or a data feed
        pattern_chars = "IXWXB"
        pattern = random.choice(pattern_chars)

        self.full_history.append({
            "timestamp": candle.get('timestamp', datetime.now().timestamp()),
            "pattern": pattern,
            "open": candle['open'],
            "high": candle['high'],
            "low": candle['low'],
            "close": candle['close']
        })

streamer = PatternStreamer()

# Seed some mock data for testing
for i in range(2000):
    price = 1.1000 + (random.random() - 0.5) * 0.1
    streamer.add_candle({
        "open": price,
        "high": price + 0.0010,
        "low": price - 0.0010,
        "close": price + (random.random() - 0.5) * 0.0005,
    })

@app.post("/api/scan_sequence")
async def scan_sequence(request: dict):
    """
    Input: { "sequence": "IXwXB", "lookback": 1000 }
    Output: List of occurrences and what happened next
    """
    query_seq = request.get("sequence", "").upper()
    limit = request.get("lookback", 1000)

    if not query_seq:
        raise HTTPException(status_code=400, detail="Sequence required")

    df = pd.DataFrame(streamer.full_history)
    if df.empty:
        return {"results": []}

    # Find indices where sequence matches
    matches = []
    seq_len = len(query_seq)

    # We use the 'pattern' column to find sequences
    patterns = df['pattern'].tolist()

    for i in range(len(patterns) - seq_len - 5):
        window = "".join(patterns[i : i + seq_len])
        if window == query_seq:
            # Look ahead 5 bars
            future = df.iloc[i + seq_len : i + seq_len + 5]

            # Calculate result (e.g., did price go up or down?)
            start_price = df.iloc[i + seq_len]['open']
            end_price = future.iloc[-1]['close'] if not future.empty else start_price
            pnl = (end_price - start_price) / start_price

            matches.append({
                "found_at": df.iloc[i]['timestamp'],
                "outcome_5bars": round(pnl * 10000, 1), # Pips
                "high_reached": future['high'].max() if not future.empty else 0,
                "low_reached": future['low'].min() if not future.empty else 0
            })

    return {
        "query": query_seq,
        "matches_found": len(matches),
        "avg_pnl": round(sum(m['outcome_5bars'] for m in matches) / len(matches), 2) if matches else 0,
        "results": matches[-20:] # Return last 20 matches
    }

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8000)
