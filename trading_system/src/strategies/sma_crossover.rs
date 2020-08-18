use super::TradingStrategy;

use ta::indicators::SimpleMovingAverage;
use ta::Next;

#[derive(Debug)]
pub struct SMACrossover {
    strategy_settings: Vec<f64>,
    short_sma: SimpleMovingAverage,
    long_sma: SimpleMovingAverage,
}

impl TradingStrategy for SMACrossover {
    fn new(strategy_settings: Vec<f64>) -> Self {
        let short_lookback = strategy_settings[0] as u32;
        let long_lookback = strategy_settings[1] as u32;

        println!(
            "Constructing SMACrossover with short_lookback: {}, and long_lookback: {}",
            short_lookback, long_lookback
        );

        SMACrossover {
            strategy_settings,
            short_sma: SimpleMovingAverage::new(short_lookback).unwrap(),
            long_sma: SimpleMovingAverage::new(long_lookback).unwrap(),
        }
    }
    fn run(&mut self, ohlc: &Vec<f64>) -> i8 {
        let curr_short_sma = self.short_sma.next(ohlc[4]);
        let curr_long_sma = self.long_sma.next(ohlc[4]);
        let mut current_signal = 0;
        if curr_short_sma > curr_long_sma {
            current_signal = 1;
        } else if curr_short_sma < curr_long_sma {
            current_signal = -1;
        }

        return current_signal;
    }

    fn to_string(&self) -> String {
        return format!("{:?}", self.strategy_settings);
    }
}
