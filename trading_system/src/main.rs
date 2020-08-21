mod binance_interface;
mod trading_strategies;
mod binance_structs;
mod helpers;
mod strategies;

use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use std::thread;
use std::io::Write;
use std::collections::HashMap;
use std::collections::VecDeque;
use binance_structs::{ReceivedData, MarketRequest};
use helpers::{epoch_ms};
use chrono::prelude::*;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader};
use std::env;
use serde_json::json;


fn main() -> Result<(), Box<dyn std::error::Error>> {

    // global vars
    let mut diagnostic = false;

    // command line args
    let args: Vec<String> = env::args().collect();

    if args.len() != 0 {
        // diagnostic flag
        if args.contains(&"diagnostic".to_string()) {
            diagnostic = true;
        }
    }

    if diagnostic {
        println!("DIAGNOSTIC MODE IS ON.");
    } else {
        println!("LIVE MODE IS ON. DIAGNOSTIC MODE IS OFF.");
    }

    // tx/rx for init
    let (init_tx, init_rx): (Sender<bool>, Receiver<bool>) = mpsc::channel();
    let init_tx2 = init_tx.clone();
    let init_tx3 = init_tx.clone();
    let init_tx4 = init_tx.clone();

    // tx/rx for human-readable logs
    let (humanlog_tx, humanlog_rx): (Sender<String>, Receiver<String>) = mpsc::channel();
    let humanlog_tx2 = humanlog_tx.clone();
    let humanlog_tx3 = humanlog_tx.clone();

    // tx/rx for file logs
    let (filelog_tx, filelog_rx): (Sender<HashMap<String, String>>, Receiver<HashMap<String, String>>) = mpsc::channel();

    // tx/rx for trades out
    let (marketreq_tx, marketreq_rx): (Sender<MarketRequest>, Receiver<MarketRequest>) = mpsc::channel();

    // tx/rx for trades confirm
    let (reqconfirm_tx, reqconfirm_rx): (Sender<bool>, Receiver<bool>) = mpsc::channel();

    // tx/rx for klines update line
    let (kline_tx, kline_rx): (Sender<ReceivedData>, Receiver<ReceivedData>) = mpsc::channel();
    let kline_tx2 = kline_tx.clone();
    let kline_tx3 = kline_tx.clone();

    // tx/rx for command lines
    let (cmd_tx, cmd_rx): (Sender<String>, Receiver<String>) = mpsc::channel();

    // thread(s) to pull live webstream data from binance
    let _klines_thread1 = thread::Builder::new().name("klines_data_thread".to_string()).spawn (move || {
        binance_interface::live_binance_stream("ethusdt@kline_1m", &kline_tx, &init_tx, binance_structs::StreamType::KLine);
    });
    let _klines_thread2 = thread::Builder::new().name("klines_data_thread".to_string()).spawn (move || {
        binance_interface::live_binance_stream("ltcusdt@kline_1m", &kline_tx2, &init_tx3, binance_structs::StreamType::KLine);
    });
    let _klines_thread3 = thread::Builder::new().name("klines_data_thread".to_string()).spawn (move || {
        binance_interface::live_binance_stream("btcusdt@kline_1m", &kline_tx3, &init_tx4, binance_structs::StreamType::KLine);
    });

    // thread for writing discord output(human friendly)
    let _humanlog_thread = thread::Builder::new().name("humanlog_thread".to_string()).spawn (move || {
        let mut last_timestamp = epoch_ms();
        let delta_time = 60000;
        let mut webhook_logs: VecDeque<String> = VecDeque::new();
        let mut humanlog_iter = humanlog_rx.try_iter();
        let webhook_url = "https://discordapp.com/api/webhooks/744067382850486372/eMESqfoTfgc1tYvkF9smKdkkyL5W1bMPn7cT_e1R-rtVXz-xfXnGO0TW5gDe7z0Lvy0U";
        loop {
            let next_data = humanlog_iter.next();
            let local_now: DateTime<Local> = Local::now();
            if next_data.is_some() {
                println!("{} ms until webhook write. ", last_timestamp as i64+delta_time as i64-epoch_ms() as i64);
                let raw_str = next_data.unwrap();
                webhook_logs.push_back(raw_str.clone());
            } 
            
            // write to discord if delta_t passed
            if epoch_ms() >= last_timestamp + delta_time {
                while webhook_logs.len() > 0 {
                    println!("webhook_logs: {:?}", webhook_logs);
                    let mut joined = String::new();
                    let mut total_length = 0;
                    loop {
                        if webhook_logs.len() == 0 {
                            break;
                        }

                        if webhook_logs[0].chars().count() + total_length >= 1500 {
                            break;
                        } else {
                            total_length += webhook_logs[0].chars().count();
                            joined = format!("{}\n{}", joined, webhook_logs.pop_front().unwrap());
                        }
                    }

                    if diagnostic {
                        println!("diagnostic print on. ");
                        joined = format!("```fix\n{}\n{}```", local_now, joined);
                    } else {
                        joined = format!("```\n{}\n\n{}```", local_now, joined);
                    }

                    // discord webhook write
                    let mut message = HashMap::new();
                    message.insert("content", joined);
                    let response = binance_interface::json_rest_req(webhook_url.to_string(), "post".to_string(), message);
                    println!("discord webhook response: {}", response);
                }
                last_timestamp += delta_time;
            }
        }
    }); 

    // thread for writing file output(machine friendly)
    // writes one json object per line. 
    let _filelog_thread = thread::Builder::new().name("filelog_thread".to_string()).spawn (move || {
        let log_ts: DateTime<Local> = Local::now();
        let file_name = format!("../logs/{}.txt", log_ts);
        let mut log_file = OpenOptions::new().append(true).create(true).open(file_name).unwrap();
        let mut filelog_iter = filelog_rx.try_iter();
        loop {
            let next_data = filelog_iter.next();
            if next_data.is_some() {
                let mut data_to_write = next_data.unwrap().clone();
                data_to_write.insert("timestamp".to_string(), format!("{}", epoch_ms()));
                let str_write = json!(data_to_write).to_string();
                let write_status = log_file.write(str_write.as_bytes());
                if write_status.is_err() {
                    let _ = humanlog_tx2.send("warning: errors with writing to log file".to_string());
                }
            }
        }
    }); 

    // thread for sending/processing market requests
    let _marketreq_thread = thread::Builder::new().name("marketreq_thread".to_string()).spawn (move || {
        loop {
            let mut marketreq_iter = marketreq_rx.try_iter();
            loop {
                let next_data = marketreq_iter.next();
                if !next_data.is_none() && !diagnostic {
                    let result = binance_interface::binance_trade_api(next_data.unwrap());
                    println!("Printing API result from market order.");
                    println!("{}", result);
                    if !result["status"].as_str().is_none() {
                        let filled_status: String = result["status"].as_str().unwrap().parse().unwrap();
                        if filled_status == "FILLED".to_string() {
                            let _ = humanlog_tx3.send("The order has been filled.".to_string());
                        } else {
                            let _ = humanlog_tx3.send("The order has not been filled for some reason.".to_string());
                        }
                    } 
                    let log_str = format!("trading_result: {}", result.to_string());
                    let _ = humanlog_tx3.send(log_str);
                    let _ = reqconfirm_tx.send(true);
                } else if !next_data.is_none() && diagnostic{
                    let _ = reqconfirm_tx.send(true);
                }
            }
        }
    });
    
    // action thread(main trading system)
    let _action_thread = thread::Builder::new().name("action_data_thread".to_string()).spawn(move || {

        /*
            Following code initializes variables.
        */
        // process flags
        let mut running = false;

        // generate/initializeportfolio management variables
        let number_algos = 1;
        let mut ohlc_history: Vec<Vec<Vec<f64>>> = Vec::new();
        let mut algo_status: Vec<i32> = vec![0; number_algos];
        let capital_split = vec![1.0];
        let symbols_interest = [
            "USDT".to_string(), 
            "ETH".to_string(), 
            "BTC".to_string(),
            "LTC".to_string(),
        ];
        let mut ticker_list = Vec::new();
        for i in 1..symbols_interest.len() {
            let ticker = format!("{}{}", symbols_interest[i], symbols_interest[0]);
            ticker_list.push(ticker);
        }
        let mut stepsize: Vec<f64> = vec![-1.0; symbols_interest.len()];
        let mut min_notional: Vec<f64> = vec![-1.0; symbols_interest.len()];
        let mut previous_signals: Vec<Vec<i32>> = vec![vec![-2; number_algos]; ticker_list.len()];
        let mut p_data: Vec<Vec<Vec<f64>>> = vec![vec![Vec::new(); number_algos]; ticker_list.len()];

        // settings(numerical only)
        let mut settings = HashMap::new();
        settings.insert("ohlc_period", 60 * 1000);
        settings.insert("max_lookback_ms", settings["ohlc_period"] * 24 * 60);

        // generate stepsize and min_notional
        let exchange_info = binance_interface::binance_rest_api("exchange_info", epoch_ms(), "");
        let symbols_arr = exchange_info["symbols"].as_array().unwrap();

        // populate USDT min_notional and stepsize with 0.01, as this info will never be used
        stepsize[0] = 0.01;
        min_notional[0] = 0.01;

        for asset in symbols_arr {
            let symbol = asset["baseAsset"].as_str().unwrap().to_string();
            let quote_asset = asset["quoteAsset"].as_str().unwrap().to_string();
            if symbols_interest.iter().position(|x| x == &symbol) != None && quote_asset == "USDT".to_string() {
                println!("symbol: {}", symbol);
                let index = symbols_interest.iter().position(|x| x == &symbol).unwrap();
                println!("index: {}", index);
                for filter in asset["filters"].as_array().unwrap() {
                    if filter["filterType"].as_str().unwrap() == "LOT_SIZE" {
                        stepsize[index] = filter["stepSize"].as_str().unwrap().parse().unwrap();
                    } else if filter["filterType"].as_str().unwrap() == "MIN_NOTIONAL" {
                        min_notional[index] = filter["minNotional"].as_str().unwrap().parse().unwrap();
                    }
                }
            }
        }

        println!("stepsize: {:?}", stepsize);
        println!("min_notional: {:?}", min_notional);

        // check if any values are unpopulated
        if stepsize.contains(&-1.0) {
            let _ = humanlog_tx.send("One or more elements of stepsize was not calculated. Panicking.".to_string());
            panic!("One or more elements of stepsize was not calculated.");
        }

        if min_notional.contains(&-1.0) {
            let _ = humanlog_tx.send("One or more elements of min_notional was not calculated. Panicking.".to_string());
            panic!("One or more elements of min_notional was not calculated.");
        }

        println!("Action initialization successful!");
        init_tx2.send(true).unwrap();

        // main loop
        loop {
            // system time 
            let time_now = epoch_ms();

            // check if command exists
            let mut command_good = true;
            let command = match cmd_rx.try_recv() {
                Ok(data) => data,
                Err(_) => {
                    command_good = false;
                    "error".to_string()
                }
            };

            // commands list 
            if command_good {
                if command == "start" {
                    running = true;
                } else if command == "stop" {
                    running = false;
                } else if command == "autostart" {
                    // COPYPASTE OF FETCHING PREDATA
                    println!("fetching predata...");
                    let api_limit = 500;
                    let end_window = epoch_ms();
                    
                    for ticker in ticker_list.iter() {
                        println!("fetching predata for {}...", ticker);
                        let start_window = end_window - settings["max_lookback_ms"];
                        let mut end_chunk = end_window;
                        let mut ticker_ohlc = Vec::new();
                        while end_chunk >= start_window {
                            let mut new_ohlcs = binance_interface::fetch_klines(&ticker, end_chunk, api_limit);
                            let mut swap = Vec::new();
                            swap.append(&mut new_ohlcs);
                            swap.append(&mut ticker_ohlc);
                            ticker_ohlc = swap.clone();
                            end_chunk -= api_limit * 60 * 1000;
                        }
                        ohlc_history.push(ticker_ohlc);
                    }

                    let _ = humanlog_tx.send("predata: finished fetching predata.".to_string());
                    println!("finished with fetching predata.");

                    // fetch first previous_signals
                    for ticker_i in 0..ticker_list.len() {
                        let (signals, _p_data_temp) = trading_strategies::master_strategy(&ohlc_history[ticker_i], &p_data[ticker_i], &humanlog_tx);
                        previous_signals[ticker_i] = signals;
                    }

                    running = true;
                } else if command == "newlistenkey" {
                    binance_interface::binance_rest_api("new_listenkey", time_now, "");
                } else if command == "displayaccountinfo" {
                    println!("{}", binance_interface::binance_rest_api("get_accountinfo", time_now, "").to_string());
                } else if command == "testping" {
                    binance_interface::binance_rest_api("test_ping", time_now, "");
                } else if command == "exchangeinfo" {
                    let exchange_info = binance_interface::binance_rest_api("exchange_info", epoch_ms(), "");
                    println!("exchange_info: {}", exchange_info);
                } else if command == "testtime" {
                    binance_interface::binance_rest_api("test_time", time_now, "");
                } else if command == "selltousdt" {
                    // WARNING: untested

                    // check account info and sells every asset to USDT
                    let account_info = binance_interface::binance_rest_api("get_accountinfo", time_now, "");
                    println!("account_info: {}", account_info);
                    
                    let mut i = 0;
                    loop {
                        if account_info["balances"][i].is_null() {
                            break;
                        } else {
                            println!("{}", account_info["balances"][i]);
                            let balance: f64 = account_info["balances"][i]["free"].as_str().unwrap().parse().unwrap();
                            let ticker_sell: String = account_info["balances"][i]["asset"].as_str().unwrap().to_string();
                            let symbol = format!("{}USDT", ticker_sell);
                            if symbol != "USDTUSDT" && balance != 0.0 {
                                let mut amt_to_sell:f64 = account_info["balances"][i]["free"].as_str().unwrap().parse().unwrap();
                                println!("original amt_to_sell: {}", amt_to_sell);
                                amt_to_sell = amt_to_sell - (amt_to_sell % 0.00001);
                                println!("final amt_to_sell: {}", amt_to_sell);
                                println!("Attempting to sell {} amount of {}...", amt_to_sell, account_info["balances"][i]["asset"]);
                                let request = MarketRequest {
                                    symbol: symbol, 
                                    side: "SELL".to_string(), 
                                    timestamp: time_now,
                                    quantity: amt_to_sell,
                                    quote_order_qty: -1.0,
                                };
                                let response = binance_interface::binance_trade_api(request);
                                if let Some(field) = response.get("status") {
                                    if field == "filled" {
                                        println!("Order was filled.");
                                    } else {
                                        println!("Order was not filled. Status response: {}", field);
                                    }
                                } else {
                                    println!("Something went wrong. Response was: {}", response);
                                }
                            }
                        }
                        i += 1;
                    }
                } else if command == "fetchpredata" {
                    println!("fetching predata...");
                    let api_limit = 500;
                    let end_window = epoch_ms();
                    
                    for ticker in ticker_list.iter() {
                        println!("fetching predata for {}...", ticker);
                        let start_window = end_window - settings["max_lookback_ms"];
                        let mut end_chunk = end_window;
                        let mut ticker_ohlc = Vec::new();
                        while end_chunk >= start_window {
                            let mut new_ohlcs = binance_interface::fetch_klines(&ticker, end_chunk, api_limit);
                            let mut swap = Vec::new();
                            swap.append(&mut new_ohlcs);
                            swap.append(&mut ticker_ohlc);
                            ticker_ohlc = swap.clone();
                            end_chunk -= api_limit * 60 * 1000;
                        }
                        ohlc_history.push(ticker_ohlc);
                    }
                    
                    let _ = humanlog_tx.send("predata: finished fetching predata.".to_string());
                    println!("finished with fetching predata.");
                } else if command == "fetchvars" {
                    println!("fetching variables...");
                    algo_status = vec![];
                    let var_file = File::open("../var_files.txt").unwrap();
                    let mut reader = BufReader::new(var_file);
                    let mut read_success = true;
                    
                    // read number of algorithms
                    let mut line = String::new();
                    let n_status = reader.read_line(&mut line);
                    if n_status.is_err() {
                        read_success = false;
                    } else {
                        let n: u64 = line.trim().parse().unwrap();
                        // read in algo_status
                        for _i in 0..n {
                            let mut line = String::new();
                            let line_status = reader.read_line(&mut line);
                            if !line_status.is_err() {
                                let status : i32 = line.trim().parse().unwrap();
                                algo_status.push(status);
                            } else {
                                read_success = false;
                                break;
                            }
                            
                        }
                        
                    }
                    
                    if read_success {
                        println!("done with fetching variables.");
                    } else {
                        let _ = humanlog_tx.send("Error reading from variable file.".to_string());
                        println!("Something went wrong reading the variable file. Maybe the format is wrong?");
                    }
                    
                } else if command == "storevars" {
                    println!("writing variables...");
                    let mut log_file = OpenOptions::new()
                        .read(true)
                        .write(true)
                        .create(true)
                        .open("../var_files.txt")
                        .unwrap();
                    let n: u64 = algo_status.len() as u64;
                    let mut write_str = format!("{}\n", n);
                    for i in 0..n {
                        write_str = format!("{}{}\n", write_str, algo_status[i as usize]);
                    }
                    log_file.write_all(write_str.as_bytes()).unwrap();
                    println!("done with writing variables.");
                } else if command == "displayvars" {
                    println!("n: {}", algo_status.len());
                    println!("algostatus: {:?}", algo_status);
                    println!("stepsize: {:?}", stepsize);
                    println!("min_notional: {:?}", min_notional);
                    println!("previous_signals: {:?}", previous_signals);
                    println!("running: {}", running);
                    println!("diagnostic: {}", diagnostic);
                    println!("\n done with printing variables.");
                } else if command == "ordertest" {
                    let signal = 1;
                    if signal == 1 {
                        let request = MarketRequest {
                            symbol: "LTCUSDT".to_string(), 
                            side: "BUY".to_string(), 
                            timestamp: epoch_ms(),
                            quantity: -1.0,
                            quote_order_qty: 10.0, 
                        };
                        let _ = humanlog_tx.send(format!("requesting trade: {}", request.clone().to_string()));
                        println!("requesting trade: {}", request.clone().to_string());
                        marketreq_tx.send(request.clone()).unwrap();
                    } else {
                        let request = MarketRequest {
                            symbol: "LTCUSDT".to_string(), 
                            side: "SELL".to_string(), 
                            timestamp: epoch_ms(),
                            quantity: 0.0,
                            quote_order_qty: -1.0,
                        };
                        let _ = humanlog_tx.send(format!("requesting trade: {}", request.clone().to_string()));
                        marketreq_tx.send(request.clone()).unwrap();
                    }
                    println!("request sent.");

                    // check to make sure that the trade went through
                    loop {
                        let req_status = reqconfirm_rx.recv().unwrap();
                        if req_status {
                            println!("req_status true came in.");
                            break;
                        }
                    }

                } 
            }

            // main trade/pm logic
            if running {
                // receive market data(trading only for now) and append to past list of trades
                let mut kline_iter = kline_rx.try_iter();
                let mut kline_valid = -1;
                loop {
                    let next_data = kline_iter.next();
                    if !next_data.is_none() {
                        let raw_kline = next_data.unwrap();
                        let kline = raw_kline.as_kline();
                        if kline.closed {
                            println!("kline.symbol: {}", kline.symbol);
                            println!("ticker_list: ");
                            for ticker in &ticker_list {
                                println!("{}", ticker);
                            }
                            let index = ticker_list.iter().position(|x| x == &kline.symbol).unwrap();
                            let append_arr = vec![kline.open, kline.high, kline.low, kline.close, kline.quantity];
                            ohlc_history[index].push(append_arr);
                            kline_valid = index as i64;

                            // logging
                            let log_str = format!("new_ohlc: {} {} {} {} {} {}", kline.symbol, kline.open, kline.high, kline.low, kline.close, kline.quantity);
                            let _ = humanlog_tx.send(log_str);
                        }
                    } else {
                        break;
                    }
                }
                
                if kline_valid != -1 {
                    println!("kline is valid. running trading logic.");
                    let ticker_i = kline_valid as usize;
                    // slice to relevant part
                    let limit_len = (settings["max_lookback_ms"] / settings["ohlc_period"]) as usize;

                    println!("On ticker: {}", ticker_list[ticker_i]);
                    if ohlc_history[ticker_i].len() >= limit_len {
                        ohlc_history[ticker_i] = ohlc_history[ticker_i][ohlc_history[ticker_i].len()-limit_len..].to_vec();
                        
                        // call master strategy
                        // keep in mind, signals returned direct from the function is either 0 or 1. This is different from algo_status, where
                        // the numbers denote which currency the algo is playing.
                        let (signals, new_p_data) = trading_strategies::master_strategy(&ohlc_history[ticker_i], &p_data[ticker_i], &humanlog_tx);
                        p_data[ticker_i] = new_p_data;

                        // logging real quick
                        let _ = humanlog_tx.send(format!("update: on ticker {}", ticker_list[ticker_i]));
                        let _ = humanlog_tx.send(format!("algo_status: {:?}", &algo_status));
                        
                        // process each signal
                        for (i, signal) in signals.iter().enumerate() {
                            println!("Current algo play is {}. ", algo_status[i]);
                            println!("Algorithm returned {} indicator.", signal);
                            /* 
                                action_condition:
                                    1. algorithm wants to sell out. In this case, check if the currency that the algo is current in 
                                        is the same as the current ticker. In that case, the sell signal is valid. 
                                    2. algorithm wants to buy in. Simply check that the algo is free and then buy in.
                                signal_diff_condition (CURRENTLY NOT IMPLEMENTED): 
                                    1. Only take action if the generated signal is different than the previous signal.
                            */
                            let action_condition = ((signal == &0 && &algo_status[i] != &0) && &algo_status[i]-1 == ticker_i as i32) 
                            || (signal == &1 && &algo_status[i] == &0);
                            let signal_diff_condition = signal != &previous_signals[ticker_i][i];
                            println!("action_condition: {}", action_condition);
                            println!("signal_diff_condition: {}", signal_diff_condition);
                            let _ = humanlog_tx.send(format!("conditions: action_condition: {} || signal_diff_condition: {}", action_condition, signal_diff_condition));
                            if action_condition  {
                                println!("signal contradicts status, taking action.");

                                // empty rx for trade confirm so only data in pipe is from the request we're about to send. 
                                loop {
                                    let status = reqconfirm_rx.try_iter().next();
                                    if status.is_none() {
                                        break;
                                    }
                                }

                                // calculate balances
                                // fetch account information and calculate relative split to put into play
                                let mut balances = vec![-1.0; symbols_interest.len()];
                                // calculate balance for each symbol in symbols_interest
                                let account_info = binance_interface::binance_rest_api("get_accountinfo", time_now, "");
                                let mut j = 0;
                                loop {
                                    if account_info["balances"][j].is_null() {
                                        break;
                                    } else {
                                        let balance: f64 = account_info["balances"][j]["free"].as_str().unwrap().parse().unwrap();
                                        let balance_ticker: String = account_info["balances"][j]["asset"].as_str().unwrap().to_string();
                                        for k in 0..symbols_interest.len() {
                                            if balance_ticker == symbols_interest[k] {
                                                balances[k] = balance;
                                            }
                                        }
                                    }
                                    j += 1;
                                }
                                println!("listing calculated balances: {:?}", balances);
                                
                                // log balance
                                let _ = humanlog_tx.send(format!("tickers: {:?}", symbols_interest));
                                let _ = humanlog_tx.send(format!("calculated_balance: {:?}", balances));

                                // calculate relative split
                                let mut total_percent = 0.0;
                                for (j, status) in algo_status.iter().enumerate() {
                                    if status == &algo_status[i] {
                                        total_percent += capital_split[j];
                                    }
                                }
                                let relative_split = capital_split[i] / total_percent;

                                if balances[algo_status[i] as usize] == -1.0 {
                                    let _ = humanlog_tx.send("warning: invalid balance. continuing to next signal in loop.".to_string());
                                    continue;
                                }

                                // if signal is 0(back to USDT), calculate amount to sell.
                                // if signal is positive(into a currency), calculate USDT amount then multiply by price
                                let mut amt = relative_split * balances[algo_status[i] as usize];
                                // amt processing
                                if signal != &0 {
                                    if amt <= min_notional[ticker_i] {
                                        println!("amount is less than min_notional. breaking out of loop iter.");
                                        let _ = humanlog_tx.send("warning: amount is less than min_notional. breaking out of loop iter.".to_string());
                                        break;
                                    }
                                } else {
                                    amt -= amt % stepsize[ticker_i];
                                }
                                println!("final amt: {}", amt);

                                if signal != &0 {
                                    let request = MarketRequest {
                                        symbol: ticker_list[ticker_i].to_string(), 
                                        side: "BUY".to_string(), 
                                        timestamp: epoch_ms(),
                                        quantity: -1.0,
                                        quote_order_qty: amt, 
                                    };
                                    let _ = humanlog_tx.send(format!("requesting trade: {}", request.clone().to_string()));
                                    marketreq_tx.send(request.clone()).unwrap();
                                    algo_status[i] = (ticker_i + 1) as i32;
                                } else {
                                    let request = MarketRequest {
                                        symbol: ticker_list[ticker_i].to_string(), 
                                        side: "SELL".to_string(), 
                                        timestamp: epoch_ms(),
                                        quantity: amt,
                                        quote_order_qty: -1.0,
                                    };
                                    let _ = humanlog_tx.send(format!("requesting trade: {}", request.clone().to_string()));
                                    marketreq_tx.send(request.clone()).unwrap();
                                    algo_status[i] = 0;
                                }

                                // check to make sure that the trade went through
                                loop {
                                    let req_status = reqconfirm_rx.recv().unwrap();
                                    if req_status {
                                        break;
                                    }
                                }
                            }
                        }

                        println!("previous_signals before: ");
                        println!("{:?}", previous_signals);
                        // update previous signal
                        previous_signals[ticker_i] = signals;

                        println!("previous_signals after: ");
                        println!("{:?}", previous_signals);
                    }
                }
            }
        }
    });

    // check if initialization complete
    let mut counter = 0;
    loop {
        let received = init_rx.recv().unwrap();
        if received == true {
            counter += 1;
        }
        if counter >= 3 {
            break;
        }
    }
    println!("Initialization finished!");
    
    // shell loop
    loop {

        // get command
        print!("Jane >>>");
        std::io::stdout().flush().unwrap();
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        
        // split into parts and generate argument vector
        let mut parts = input.trim().split_whitespace();

        let command = match parts.next() {
            Some(command) => command,
            None => "no command found"
        };
        if command == "no command found" {
            continue;
        }
        let mut args: Vec<&str> = Vec::new();
        for arg in parts {
            args.push(arg);
        }

        if command == "quit" || command == "exit" {
            break;
        }
        
        // send command to action thread
        cmd_tx.send(command.to_string()).unwrap();
        
    }


    Ok(())
}
