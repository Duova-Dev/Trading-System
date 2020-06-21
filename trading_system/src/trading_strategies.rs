pub mod strategies {
    use std::collections::HashMap;
    use serde_json::value::Value;

    pub enum TradeOrder {
        Limit(LimitRequest),
        Market(MarketRequest),
    }

    pub struct LimitRequest {
        symbol: String,
        side: String,
        timestamp: u64,
        timeInForce: String,
        quantity: f64,
        price: u64,
    }

    pub struct MarketRequest {
        symbol: String,
        side: String,
        timestamp: u64,
        quantity: f64,
    }

    // below is not supported yet
    struct StopLossRequest {
        symbol: String,
        side: String,
        timestamp: u64,
    }

    struct StopLossLimitRequest {
        symbol: String,
        side: String,
        timestamp: u64,
    }

    struct TakeProfitRequest {
        symbol: String,
        side: String,
        timestamp: u64,
    }

    struct TakeProfitLimitRequest {
        symbol: String,
        side: String,
        timestamp: u64,
    }

    fn sma_reversion(timestamp: u64) -> Vec<TradeOrder> {
        println!("sma_reversion is running!");
        let sample_order = TradeOrder::Market( MarketRequest {
            symbol: "btcusdt".to_string(),
            side: "BUY".to_string(),
            timestamp: timestamp,
            quantity: 1.0,
        });
        return vec![sample_order];
    }

    pub fn master_strategy(algos_to_run: Vec<u32>, 
        trades_in_window: Vec<Value>,
        timestamp: u64, 
        ) -> HashMap<u32, Vec<TradeOrder>> {

        let mut id_to_algo = HashMap::new();
        id_to_algo.insert(0, sma_reversion);
        id_to_algo[&0];

        let mut orders_to_return = HashMap::new();
        for algo_id in algos_to_run {
            orders_to_return.insert(algo_id, id_to_algo[&algo_id](timestamp));
        }

        return orders_to_return;
    }

}