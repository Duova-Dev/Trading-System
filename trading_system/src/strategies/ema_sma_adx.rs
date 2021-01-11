use ta::indicators::ExponentialMovingAverage;
use ta::indicators::SimpleMovingAverage;
use ta::Next;

use super::TradingStrategy;
use crate::strategies::indicators::adx_indicator::AverageDirectionalMovementIndex;
use crate::strategies::indicators::NextI;

pub struct EMASMAADX {
    strategy_settings: Vec<f64>,
    short_ema: ExponentialMovingAverage,
    long_sma: SimpleMovingAverage,
    adx_indicator: AverageDirectionalMovementIndex,
}

impl TradingStrategy for EMASMAADX {
    fn new(strategy_settings: Vec<f64>) -> Self {
        EMASMAADX {
            strategy_settings: strategy_settings.clone(),
            short_ema: ExponentialMovingAverage::new(strategy_settings[0] as u32).unwrap(),
            long_sma: SimpleMovingAverage::new(strategy_settings[1] as u32).unwrap(),
            adx_indicator: AverageDirectionalMovementIndex::new(
                strategy_settings[2] as u64,
                strategy_settings[3] as u64,
            ),
        }
    }
    fn run(&mut self, ohlc: &Vec<f64>) -> i8 {
        let curr_short_ema = self.short_ema.next(ohlc[4]);
        let curr_long_sma = self.long_sma.next(ohlc[4]);
        let curr_adx = self.adx_indicator.next(ohlc[4]);

        let mut current_signal = 0;
        if curr_short_ema > curr_long_sma && curr_adx > 25f64 {
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
