

use std::collections::HashMap;
use serde_json::value::Value;
use crate::binance_structs;
use crate::binance_structs::{OccuredTrade};


fn sma_reversion(trades: &Vec<Vec<f64>>) -> i32 {
    println!("sma_reversion is running!");
    let short_period = 576;
    let long_period = 24 * 60;

    let trades_len = trades.len();

    let mut long_sum = 0.0;
    for i in trades_len-long_period..trades_len{
        long_sum += trades[i][3];
    }
    let mut short_sum = 0.0;
    for i in trades_len-short_period..trades_len{
        short_sum += trades[i][3];
    }

    let long_avg = long_sum / long_period as f64;
    let short_avg = short_sum / short_period as f64;

    if long_avg > short_avg {
        println!("long_avg: {}, short_avg: {}. returning sell signal.", long_avg, short_avg);
        return 0;
    } else if long_avg < short_avg {
        println!("long_avg: {}, short_avg: {}. returning buy signal.", long_avg, short_avg);
        return 1;
    } else {
        println!("long_avg: {}, short_avg: {}. returning neutral signal.", long_avg, short_avg);
        return 0;
    }
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

    let mut strategies_list = vec![sma_reversion];

    let mut signals = Vec::new();
    for strategy in strategies_list {
        signals.push(strategy(trades));
    }

    return signals;
}
