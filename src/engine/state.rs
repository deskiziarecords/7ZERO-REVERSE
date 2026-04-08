use std::collections::VecDeque;
use crate::types::types::Candle;

pub struct State {
    pub candles: VecDeque<Candle>,
    pub max_size: usize,
}

impl State {
    pub fn new(size: usize) -> Self {
        Self {
            candles: VecDeque::with_capacity(size),
            max_size: size,
        }
    }

    pub fn push(&mut self, candle: Candle) {
        if self.candles.len() == self.max_size {
            self.candles.pop_front();
        }
        self.candles.push_back(candle);
    }

    pub fn get_candles(&self) -> Vec<Candle> {
        self.candles.iter().cloned().collect()
    }
}
