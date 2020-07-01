

use std::collections::HashMap;
use serde_json::value::Value;
use crate::binance_structs;
use crate::binance_structs::{OccuredTrade, TradeOrder};


fn sma_reversion(timestamp: u64) -> Vec<binance_structs::TradeOrder> {
    println!("sma_reversion is running!");
    let sample_order = binance_structs::TradeOrder::Market( binance_structs::MarketRequest {
        symbol: "btcusdt".to_string(),
        side: "BUY".to_string(),
        timestamp: timestamp,
        quantity: 1.0,
    });
    return vec![sample_order];
}

pub fn master_strategy(
    algos_to_run: Vec<u32>, 
    trades_in_window: Vec<OccuredTrade>,
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
