mod binance_interface;
mod trading_strategies;
mod binance_structs;
mod helpers;

use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use std::thread;
use std::io::Write;
use std::collections::HashMap;
use std::convert::TryFrom;
use serde::{Deserialize};
use serde_json::{Value};
use binance_structs::{ReceivedData, MarketRequest};
use helpers::{epoch_ms};
use chrono::prelude::*;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader};


fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    // tx/rx for init
    let (init_tx, init_rx): (Sender<bool>, Receiver<bool>) = mpsc::channel();
    let init_tx2 = init_tx.clone();
    let init_tx3 = init_tx.clone();

    // tx/rx for quit
    let (quit_tx, quit_rx): (Sender<bool>, Receiver<bool>) = mpsc::channel();

    // tx/rx for things to print
    let (print_tx, print_rx): (Sender<String>, Receiver<String>) = mpsc::channel();

    // tx/rx for things to print
    let (logging_tx, logging_rx): (Sender<String>, Receiver<String>) = mpsc::channel();
    let logging_tx1 = logging_tx.clone();

    // tx/rx for trades out
    let (marketreq_tx, marketreq_rx): (Sender<MarketRequest>, Receiver<MarketRequest>) = mpsc::channel();

    // tx/rx for user_data stream
    let (userdata_tx, userdata_rx): (Sender<ReceivedData>, Receiver<ReceivedData>) = mpsc::channel();

    // tx/rx for klines update line
    let (kline_tx, kline_rx): (Sender<ReceivedData>, Receiver<ReceivedData>) = mpsc::channel();

    // tx/rx for command lines
    let (cmd_tx, cmd_rx): (Sender<String>, Receiver<String>) = mpsc::channel();
    let cmd_tx_action = cmd_tx.clone();

    // thread to pull live webstream data from binance
    let klines_thread = thread::Builder::new().name("klines_data_thread".to_string()).spawn (move || {
        binance_interface::live_binance_stream("ethusdt@kline_1m", &kline_tx, &init_tx, binance_structs::StreamType::KLine);
    });
    
    // thread for user data
    let userdata_thread = thread::Builder::new().name("userdata_thread".to_string()).spawn (move || {
        let listen_key = binance_interface::new_listenkey(epoch_ms());
        binance_interface::live_binance_stream(&listen_key, &userdata_tx, &init_tx2, binance_structs::StreamType::UserData);
    });

    // thread for logging
    let logging_thread = thread::Builder::new().name("logging_thread".to_string()).spawn (move || {
        let local_now: DateTime<Local> = Local::now();
        let file_name = format!("../logs/{}.txt", local_now);
        let mut log_file = OpenOptions::new().append(true).create(true).open(file_name).unwrap();
        loop {
            let mut logging_iter = logging_rx.try_iter();
            loop {
                let next_data = logging_iter.next();
                if !next_data.is_none() {
                    let raw_str = next_data.unwrap();
                    let write_str = format!("{}:: {}\n", epoch_ms(), raw_str);
                    log_file.write(write_str.as_bytes()).unwrap();
                } else {
                    break;
                }
            }
        }
    });


    // thread for sending/processing market requests
    let marketreq_thread = thread::Builder::new().name("marketreq_thread".to_string()).spawn (move || {
        loop {
            let mut marketreq_iter = marketreq_rx.try_iter();
            loop {
                let next_data = marketreq_iter.next();
                if !next_data.is_none() {
                    let result = binance_interface::binance_trade_api(next_data.unwrap());
                    let log_str = format!("trading_result: {}", result.to_string());
                    logging_tx1.send(log_str);
                    println!("Printing API result from market order.");
                    println!("{}", result);
                } else {
                    break;
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
        let mut diagnostic = false;

        // convenience variables
        let mut previous_displayed_epoch: u64 = 0;

        // portfolio management data
        let mut ohlc_history: Vec<Vec<f64>> = Vec::new();
        let mut algo_status: Vec<i32> = vec![0];
        let capital_split = vec![1.0];
        let eth_stepsize = 0.00001;
        let symbols_interest = ["USDT".to_string(), "ETH".to_string()];
        let ticker = "ETHUSDT";

        // settings(numerical only)
        let mut settings = HashMap::new();
        settings.insert("ohlc_period", 60 * 1000);
        settings.insert("max_lookback_ms", settings["ohlc_period"] * 24 * 60);

        // hashmap tracking balance to be updated every time an api update comes in
        let mut balances = HashMap::new();
        // initialize balance
        let account_info = binance_interface::binance_rest_api("get_accountinfo", epoch_ms(), "");
        let received_balances_vec = account_info["balances"].as_array().unwrap();
        for asset in received_balances_vec {
            let symbol = asset["asset"].as_str().unwrap().to_string();
            let amt : f64 = asset["free"].as_str().unwrap().parse().unwrap();
            if symbols_interest.contains(&symbol) {
                balances.insert(symbol.to_string(), amt);
            }
        }
        for symbol in &symbols_interest {
            println!("{}: {}", symbol, balances[symbol]);
        }

        println!("Action initialization successful!");
        init_tx3.send(true).unwrap();

        // main loop
        loop {
            // system time 
            let time_now = epoch_ms();

            // check if command exists
            let mut command_good = true;
            let command = match cmd_rx.try_recv() {
                Ok(data) => data,
                Err(e) => {
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
                                println!("amt_to_sell % eth_stepsize: {}", amt_to_sell % eth_stepsize);
                                amt_to_sell = amt_to_sell - (amt_to_sell % eth_stepsize);
                                println!("final amt_to_sell: {}", amt_to_sell);
                                println!("Attempting to sell {} amount of {}...", amt_to_sell, account_info["balances"][i]["asset"]);
                                let request = MarketRequest {
                                    symbol: symbol, 
                                    side: "SELL".to_string(), 
                                    timestamp: time_now,
                                    quantity: amt_to_sell,
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
                } else if command == "startdiagnostic" {
                    diagnostic = true;
                } else if command == "fetchpredata" {
                    println!("fetching predata...");
                    let api_limit = 500;
                    let end_window = epoch_ms();
                    let start_window = end_window - settings["max_lookback_ms"];
                    let mut end_chunk = end_window;
                    while end_chunk >= start_window {
                        let mut new_ohlcs = binance_interface::fetch_klines(ticker, end_chunk, api_limit);
                        let mut swap = Vec::new();
                        swap.append(&mut new_ohlcs);
                        swap.append(&mut ohlc_history);
                        ohlc_history = swap.clone();
                        end_chunk -= api_limit * 60 * 1000;
                    }
                    println!("finished with fetching predata.");

                    let mut i = 0;
                    for ohlc in &ohlc_history {
                        println!("row: {} open: {} high: {} low: {} close: {} volume: {}", i, ohlc[0], ohlc[1], ohlc[2], ohlc[3], ohlc[4]);
                        i += 1;
                    }

                } else if command == "fetchvars" {
                    println!("fetching variables...");
                    algo_status = vec![];
                    let mut var_file = File::open("../var_files.txt").unwrap();
                    let mut reader = BufReader::new(var_file);
                    
                    // read number of algorithms
                    let mut line = String::new();
                    reader.read_line(&mut line);
                    let n: u64 = line.trim().parse().unwrap();
                    
                    // read in algo_status
                    for i in 0..n {
                        let mut line = String::new();
                        reader.read_line(&mut line);
                        let status : i32 = line.trim().parse().unwrap();
                        algo_status.push(status);
                    }
                    println!("done with fetching variables.");
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
                    println!("printing algostatus...");
                    for i in 0..algo_status.len() {
                        print!("{} ", algo_status[i]);
                    }
                    println!("\n done with printing variables.");
                } else if command == "test1" {
                    logging_tx.send("yes.".to_string());
                }
            }

            // main trade/pm logic
            if running {

                // check for user data updates. currently only deals with balance
                // receive balance updates(if any) and update balances
                let mut userdata_iter = userdata_rx.try_iter();
                loop {
                    let next_data = userdata_iter.next();
                    if !next_data.is_none() {
                        let userdata_val = next_data.unwrap().as_value();
                        let updated_balances = userdata_val["balances"].as_array().unwrap();
                        for asset in updated_balances {
                            let symbol = asset["asset"].as_str().unwrap().to_string();
                            let amt : f64 = asset["free"].as_str().unwrap().parse().unwrap();
                            if symbols_interest.contains(&symbol) {
                                balances.insert(symbol, amt);
                            }
                        }
                    } else {
                        break;
                    }
                }

                // receive market data(trading only for now) and append to past list of trades
                let mut kline_iter = kline_rx.try_iter();
                let mut kline_valid = false;
                loop {
                    let next_data = kline_iter.next();
                    if !next_data.is_none() {
                        let raw_kline = next_data.unwrap();
                        let kline = raw_kline.as_kline();
                        if kline.closed {
                            kline_valid = true;
                            // append to ohlc_history 
                            let append_arr = vec![kline.open, kline.high, kline.low, kline.close, kline.quantity];
                            ohlc_history.push(append_arr);
                            // logging
                            let log_str = format!("new_ohlc: {} {} {} {} {}", kline.open, kline.high, kline.low, kline.close, kline.quantity);
                            logging_tx.send(log_str);
                        }
                    } else {
                        break;
                    }
                }
                
                if kline_valid {
                    println!("kline is valid. running trading logic.");

                    // slice to relevant part
                    let limit_len = (settings["max_lookback_ms"] / settings["ohlc_period"]) as usize;
                    if ohlc_history.len() >= limit_len {
                        ohlc_history = ohlc_history[ohlc_history.len()-limit_len..].to_vec();

                        // portfolio management
                        // hashmap of algo id : capital allocation
                        let mut capital_allocation = HashMap::new();
                        capital_allocation.insert(0, 1);
                        
                        // call master strategy
                        let signals = trading_strategies::master_strategy(&ohlc_history);
                        
                        for (i, signal) in signals.iter().enumerate() {
                            println!("Current signal is {}. ", algo_status[i]);
                            println!("Algorithm returned {}.", signal);
                            if signal != &algo_status[i] {
                                println!("signal contradicts status, taking action.");
                                
                                // fetch account information and calculate relative split to put into play

                                // calculate balance for each symbol in symbols_interest
                                let account_info = binance_interface::binance_rest_api("get_accountinfo", time_now, "");

                                // logging real quick
                                let log_str = format!("account_update: {}", account_info.to_string());
                                logging_tx.send(log_str);

                                let mut j = 0;
                                let mut balances = vec![-1.0; symbols_interest.len()];
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

                                println!("listing calculated balances: ");
                                for balance in &balances {
                                    println!("{}", balance);
                                }

                                let mut total_percent = 0.0;
                                // calculate relative split
                                for (j, status) in algo_status.iter().enumerate() {
                                    if status == &algo_status[i] {
                                        total_percent += capital_split[j];
                                    }
                                }
                                let relative_split = capital_split[i] / total_percent;

                                println!("relative_split: {}", relative_split);

                                if balances[algo_status[i] as usize] == -1.0 {
                                    panic!("Invalid balance.");
                                }

                                // if signal is 0(back to USDT), calculate amount to sell.
                                // if signal is positive(into a currency), calculate USDT amount then multiply by price
                                let mut amt = relative_split * balances[algo_status[i] as usize];

                                println!("final amt: {}", amt);
                                
                                if signal != &0 {
                                    amt /= ohlc_history[ohlc_history.len()-1][3];
                                    let request_time = epoch_ms();
                                    let request = MarketRequest {
                                        symbol: ticker.to_string(), 
                                        side: "BUY".to_string(), 
                                        timestamp: request_time,
                                        quantity: amt
                                    };
                                    marketreq_tx.send(request.clone()).unwrap();
                                    let log_str = format!("market_request: {} BUY {} {}", ticker.to_string(), request_time, amt);
                                    logging_tx.send(log_str);
                                } else {
                                    let request_time = epoch_ms();
                                    let request = MarketRequest {
                                        symbol: ticker.to_string(), 
                                        side: "SELL".to_string(), 
                                        timestamp: epoch_ms(),
                                        quantity: amt
                                    };
                                    marketreq_tx.send(request.clone()).unwrap();
                                    let log_str = format!("market_request: {} SELL {} {}", ticker.to_string(), request_time, amt);
                                    logging_tx.send(log_str);
                                }
                                algo_status[i] = *signal;
                            }
                        }
                    }
                }
                // logging
                if epoch_ms() % 1000 == 0  && epoch_ms() != previous_displayed_epoch{
                    println!("Current time is: {}", epoch_ms());
                    previous_displayed_epoch = epoch_ms();
                }
            }
            if diagnostic {
                let mut userdata_iter = userdata_rx.try_iter();
                loop {
                    let next_data = userdata_iter.next();
                    if !next_data.is_none() {
                        let userdata_update = next_data.unwrap();
                        println!("userdata_update: {}", userdata_update.as_value());
                    } else {
                        break;
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
        print!("Duova Capital CLI>>>");
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
        
        // receive anything that should be printed and then print them
        let mut print_iter = print_rx.try_iter();
        loop {
            let next_print = print_iter.next();
            if next_print != None {
                println!("{}", next_print.unwrap());
            } else {
                break;
            }
        }
        
    }


    Ok(())
}
