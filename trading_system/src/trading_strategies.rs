

use std::collections::HashMap;
use serde_json::value::Value;
use crate::binance_structs;
use crate::binance_structs::{OccuredTrade};


fn sma_reversion(trades: &Vec<Vec<f64>>) -> i32 {
    println!("sma_reversion is running!");

    return 1;
}

pub fn master_strategy(
    trades: &Vec<Vec<f64>>,
) -> Vec<i32> {
    /*
        One function to call all the strategies that are needed.
        Parameters:
            trades: 
                trades that are within the window required
        Returns:
            HashMap:
                maps the ID of the algorithm to the result it returned.
    */

    let mut strategies_list = Vec::new();
    strategies_list.push(sma_reversion);

    let mut signals = Vec::new();
    for strategy in strategies_list {
        signals.push(strategy(trades));
    }

    return signals;
}
