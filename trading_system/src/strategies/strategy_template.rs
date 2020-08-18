use super::TradingStrategy;

pub struct TemplateStrategy {
    strategy_settings: Vec<f64>,
}

impl TradingStrategy for TemplateStrategy {
    fn new(strategy_settings: Vec<f64>) -> Self {
        TemplateStrategy {
            strategy_settings
        }
    }
    fn run(&mut self, ohlcs_in_window: &[Vec<f64>]) -> i8 {
        return 0;
    }

    fn to_string(&self) -> String {
        return format!("{:?}", self.strategy_settings);
    }
}
