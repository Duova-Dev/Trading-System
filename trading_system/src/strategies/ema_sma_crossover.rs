use ta::indicators::ExponentialMovingAverage;
use ta::indicators::SimpleMovingAverage;
use ta::Next;

use super::TradingStrategy;

#[derive(Debug)]
pub struct EMASMACrossover {
    strategy_settings: Vec<f64>,
    short_ema: ExponentialMovingAverage,
    long_sma: SimpleMovingAverage,
    supplemental_data: Vec<Vec<f64>>,
    supplemental_labels: Vec<String>,
}

impl EMASMACrossover {
    pub fn new(strategy_settings: Vec<f64>) -> Self {
        let short_ema_lookback = strategy_settings[0] as u32;
        let long_sma_lookback = strategy_settings[1] as u32;
        Self {
            strategy_settings,
            short_ema: ExponentialMovingAverage::new(short_ema_lookback).unwrap(),
            long_sma: SimpleMovingAverage::new(long_sma_lookback).unwrap(),
            supplemental_data: vec![vec![], vec![]],
            supplemental_labels: vec![String::from("Short EMA"), String::from("Long EMA")],
        }
    }
}

impl TradingStrategy for EMASMACrossover {
    fn run(&mut self, ohlc: &Vec<f64>) -> i8 {
        let curr_short_ema = self.short_ema.next(ohlc[4]);
        let curr_long_sma = self.long_sma.next(ohlc[4]);

        self.supplemental_data[0].push(curr_short_ema);
        self.supplemental_data[1].push(curr_long_sma);

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

    fn get_supplemental_data(&self) -> &Vec<Vec<f64>> {
        &self.supplemental_data
    }

    fn get_supplemental_labels(&self) -> &Vec<String> {
        &self.supplemental_labels
    }

}