use super::TradingStrategy;

#[derive(Debug)]
pub struct TemplateStrategy {
    strategy_settings: Vec<f64>,
    supplemental_data: Vec<Vec<f64>>,
    supplemental_labels: Vec<String>,
    strategy_name: String,
}

impl TemplateStrategy{
    pub fn new(strategy_settings: Vec<f64>) -> Self {
        Self {
            strategy_settings,
            supplemental_data: vec![],
            supplemental_labels: vec![], 
            strategy_name
        }
    }
}

impl TradingStrategy for TemplateStrategy {
    fn run(&mut self, ohlcs_in_window: &Vec<f64>) -> i8 {
        return ohlcs_in_window[0] as i8;
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

    fn get_name(&self) -> &String {
        &self.strategy_name
    }
}