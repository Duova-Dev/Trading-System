use super::TradingStrategy;

use ta::indicators::ExponentialMovingAverage;
use ta::indicators::SimpleMovingAverage;
use ta::Next;

#[derive(Debug)]
pub struct EMASMACrossover {
    strategy_settings: Vec<f64>,
    short_ema: ExponentialMovingAverage,
    long_sma: SimpleMovingAverage,
}

impl TradingStrategy for EMASMACrossover {
    fn new(strategy_settings: Vec<f64>) -> Self {
        let short_ema_lookback = strategy_settings[0] as u32;
        let long_sma_lookback = strategy_settings[1] as u32;
        EMASMACrossover {
            strategy_settings,
            short_ema: ExponentialMovingAverage::new(short_ema_lookback).unwrap(),
            long_sma: SimpleMovingAverage::new(long_sma_lookback).unwrap(),
        }
    }

    fn run(&mut self, ohlc: &Vec<f64>) -> i8 {
        let curr_short_ema = self.short_ema.next(ohlc[4]);
        let curr_long_sma = self.long_sma.next(ohlc[4]);

        let mut current_signal = 0;
        if curr_short_ema > curr_long_sma {
            current_signal = 1;
        } else if curr_short_ema < curr_long_sma {
            current_signal = -1;
        }

        current_signal
    }

    fn to_string(&self) -> String {
        return format!("{:?}", self.strategy_settings);
    }
}
