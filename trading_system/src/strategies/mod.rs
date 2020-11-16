pub mod indicators;
// pub mod sma_crossover;
pub mod ema_sma_crossover;
// pub mod adx;
//pub mod ema_sma_adx;


pub trait TradingStrategy {
    // fn new(strategy_settings: Vec<f64>) -> Self;
    fn run(&mut self, ohlcs_in_window: &Vec<f64>) -> i8;
    fn to_string(&self) -> String;
    fn get_supplemental_data(&self) -> &Vec<Vec<f64>>;
    fn get_supplemental_labels(&self) -> &Vec<String>;
}

