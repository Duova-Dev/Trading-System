


use std::sync::mpsc::Sender;
use crate::strategies::*;
use std::collections::HashMap;
use crate::strategies::ema_sma_crossover::EMASMACrossover;
use std::any::type_name;

/*
fn _sma_crossover(trades: &Vec<Vec<f64>>, _i_p_data: &Vec<f64>) -> (i32, Vec<f64>, HashMap<String, String>) {
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

fn legacy_ema_sma_crossover(trades: &Vec<Vec<f64>>, i_p_data: &Vec<f64>) -> (i32, Vec<f64>, HashMap<String, String>) {
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

    let signal;
    let mut log_map;
    if new_ema > sma {
        log_map = HashMap::new();
        log_map.insert("ema".to_string(), new_ema.to_string());
        log_map.insert("sma".to_string(), sma.to_string());
        log_map.insert("signal".to_string(), 1.to_string());
        signal = 1;
    } else if new_ema < sma {
        log_map = HashMap::new();
        log_map.insert("ema".to_string(), new_ema.to_string());
        log_map.insert("sma".to_string(), sma.to_string());
        log_map.insert("signal".to_string(), 0.to_string());
        signal = 0;
    } else {
        log_map = HashMap::new();
        log_map.insert("ema".to_string(), new_ema.to_string());
        log_map.insert("sma".to_string(), sma.to_string());
        log_map.insert("signal".to_string(), 0.to_string());
        signal = 0;
    }

    return (signal, vec![new_ema], log_map);
}
*/

fn type_of<T>(_: T) -> &'static str {
    type_name::<T>()
}

pub fn master_setup() -> Vec<Box<dyn TradingStrategy>> {
    /*
        A function to instantiate the trading strategy objects. Every object returned in the vector implements TradingStrategy, and each iteration is 
        expected to be run with master_strategy. 
        
        Arguments: 
            NA
        
        Returns: 
            Vec<Box<dyn TradingStrategy>>: a vector of the trading strategy objects, surrounded in a box. 
    */
    let mut strategy_objs: Vec<Box<dyn TradingStrategy>> = Vec::new();
    // strategy 0 - ema sma crossover @ [12 hrs, 24 hrs]
    strategy_objs.push(Box::new(EMASMACrossover::new(vec![12f64*60f64, 24f64*60f64])));

    return strategy_objs;
}

pub fn master_strategy(ohlc: &Vec<f64>, strategy_objs: &mut Vec<Box<dyn TradingStrategy>>) -> Vec<i32> {
    /*
        A function to actually run the strategies. The strategies instantiated with master_setup is passed here. This function is expected to be run 
        at every timestep. 

        Arguments: 
            ohlc: &Vec<f64> - The OHLC vector, structure specified in variables.txt 
            strategy_objects: &mut Vec<Box<dyn TradingStrategy>> - TradingStrategy objects
        
        Returns: 
            Vec<i32> - a vector of length n(number of strategies) that contains the long or sell/short signals. 
    */
    let mut signals_vec: Vec<i32> = Vec::new(); 
    let strategy_n = strategy_objs.len();
    for i in 0 .. strategy_n {
        let strategy = &mut strategy_objs[i];
        let signal = strategy.run(ohlc) as i32;
        signals_vec.push(signal);
    }
    return signals_vec;
}

pub fn grab_supp_logs(strategy_objs: &mut Vec<Box<dyn TradingStrategy>>) -> Vec<String> {
    /*
        A function to generate the logs for all of the strategies. This data includes the numbers, such as the actual EMA and SMA values. 
        It's done by mapping the supplemental labels of each strategy object to the latest timestep of all the supplemental data
        Arguments: 
            ohlc: &Vec<f64> - The OHLC vector, structure specified in variables.txt 
            strategy_objects: &mut Vec<Box<dyn TradingStrategy>> - TradingStrategy objects
        
        Returns: 
            Vec<String> - Vector of log messages
    */
    let mut final_logs: Vec<String> = Vec::new();
    for i in 0 ..strategy_objs.len() {
        let strategy = &mut strategy_objs[i];
        let labels = strategy.get_supplemental_labels();
        let data = strategy.get_supplemental_data();
        let name = type_of(&strategy);
        let mut log = format!("{}:", name);
        for j in 0 .. labels.len() {
            let piece = data[j][data[j].len()-1];
            log = [log, format!("{}-{}", labels[j], piece)].join(" ");
        }
        final_logs.push(log);
    }
    return final_logs;
}

/*
pub fn master_strategy(
    trades: &Vec<Vec<f64>>,
    incoming_p_data: &Vec<Vec<f64>>,
) -> (Vec<i32>, Vec<Vec<f64>>, Vec<HashMap<String, String>>) {
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

    let strategies_list: Vec< 
        &dyn Fn(&Vec<Vec<f64>>, &Vec<f64>) -> (i32, Vec<f64>, HashMap<String, String>)> = vec![&ema_sma_crossover];

    let mut signals = Vec::new();
    let mut p_data = Vec::new();
    let mut logs = Vec::new();
    let mut i = 0;
    for strategy in strategies_list {
        let (signal, p_data_piece, log_map) = strategy(trades, &incoming_p_data[i]);
        logs.push(log_map);
        signals.push(signal);
        p_data.push(p_data_piece);
        i += 1;
    }

    // experimental split
    // 0: ema_sma_crossover
    let settings_0 = vec![12f64*60f64, 24f64*60f64];
    let strategy_0 = EMASMACrossover::new(settings_0);

    
    return (signals, p_data, logs);
*/
