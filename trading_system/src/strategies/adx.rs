use super::TradingStrategy;

use crate::strategies::indicators::adx_indicator::AverageDirectionalMovementIndex;
use crate::strategies::indicators::NextI;

pub struct ADXTest {
    strategy_settings: Vec<f64>,
    adx_indicator: AverageDirectionalMovementIndex,
}

impl TradingStrategy for ADXTest {
    fn new(strategy_settings: Vec<f64>) -> Self {
        ADXTest {
            strategy_settings: strategy_settings.clone(),
            adx_indicator: AverageDirectionalMovementIndex::new(
                strategy_settings[0] as u64,
                strategy_settings[1] as u64,
            ),
        }
    }
    fn run(&mut self, ohlc: &Vec<f64>) -> i8 {
        let curr_adx = self.adx_indicator.next(ohlc[4]);
        // println!("{}", curr_adx);

        return if curr_adx > 25f64 { 1 } else { 0 };
    }

    fn to_string(&self) -> String {
        return format!("{:?}", self.strategy_settings);
    }
}
