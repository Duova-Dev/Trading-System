

use std::collections::HashMap;
use serde_json::value::Value;
use crate::binance_structs;
use crate::binance_structs::{OccuredTrade};
use std::sync::mpsc::{Sender, Receiver};


fn sma_crossover(trades: &Vec<Vec<f64>>, i_p_data: &Vec<f64>) -> (i32, Vec<f64>, String) {
    // NOT FUNCTIONAL
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
        println!("SMA Crossover: long_avg: {}, short_avg: {}. returning sell signal.", long_avg, short_avg);
        return (0, Vec::new(), String::new());
    } else if long_avg < short_avg {
        println!("SMA Crossover: long_avg: {}, short_avg: {}. returning buy signal.", long_avg, short_avg);
        return (1, Vec::new(), String::new());
    } else {
        println!("SMA Crossover: long_avg: {}, short_avg: {}. returning neutral signal.", long_avg, short_avg);
        return (0, Vec::new(), String::new());
    }
}

fn ema_sma_crossover(trades: &Vec<Vec<f64>>, i_p_data: &Vec<f64>) -> (i32, Vec<f64>, String) {
    let ema_lookback = 12f64 * 60f64;
    let sma_lookback = 24f64 * 60f64;
    let smoothing = 2f64;
    let trades_len = trades.len();
    let c_price = trades[trades_len-1][3];
    let mut previous_ema = 0.0;
    if i_p_data.len() != 0 {
        previous_ema = i_p_data[0];
    } else {
        for i in trades_len-ema_lookback as usize .. trades_len{
            previous_ema += trades[i][3];
        }
        previous_ema /= ema_lookback;
    }
    
    let new_ema = c_price * (smoothing / (1f64 + ema_lookback)) + previous_ema * (1f64 - (smoothing / (1f64 + ema_lookback)));
    
    let mut sma = 0.0;
    for i in trades_len-sma_lookback as usize..trades_len {
        sma += trades[i][3];
    }
    sma /= sma_lookback;

    let mut signal = 0;
    let mut log_str = String::new();
    if new_ema > sma {
        log_str = format!("EMA SMA Crossover: ema: {}, sma: {}. returning buy signal.", new_ema, sma);
        signal = 1;
    } else if new_ema < sma {
        log_str = format!("EMA SMA Crossover: ema: {}, sma: {}. returning sell signal.", new_ema, sma);
        signal = 0;
    } else {
        log_str = format!("EMA SMA Crossover: ema: {}, sma: {}. returning neutral signal.", new_ema, sma);
        signal = 0;
    }

    println!("{}", log_str);
    return (signal, vec![new_ema], log_str);
}

pub fn master_strategy(
    trades: &Vec<Vec<f64>>,
    incoming_p_data: &Vec<Vec<f64>>,
    logging_tx: &Sender<String>
) -> (Vec<i32>, Vec<Vec<f64>>) {
    /*
        One function to call all the strategies that are needed.
        Parameters:
            trades: 
                trades that are within the window required
            incoming_p_data:
                p_data that was returned from the algorithms last run
        Returns:
            HashMap:
                maps the ID of the algorithm to the result it returned.
    */

    let mut strategies_list: Vec< 
        &dyn Fn(&Vec<Vec<f64>>, &Vec<f64>) -> (i32, Vec<f64>, String)> = vec![&ema_sma_crossover];

    let mut signals = Vec::new();
    let mut p_data = Vec::new();
    let mut i = 0;
    for strategy in strategies_list {
        let (signal, p_data_piece, logging_str) = strategy(trades, &incoming_p_data[i]);
        logging_tx.send(format!("algo_logs: {}", logging_str));
        signals.push(signal);
        p_data.push(p_data_piece);
        i += 1;
    }

    return (signals, p_data);
}
