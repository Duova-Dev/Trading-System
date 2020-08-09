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
    let init_tx4 = init_tx.clone();

    // tx/rx for quit
    let (quit_tx, quit_rx): (Sender<bool>, Receiver<bool>) = mpsc::channel();

    // tx/rx for things to print
    let (print_tx, print_rx): (Sender<String>, Receiver<String>) = mpsc::channel();

    // tx/rx for things to log
    let (logging_tx, logging_rx): (Sender<String>, Receiver<String>) = mpsc::channel();
    let logging_tx1 = logging_tx.clone();

    // tx/rx for trades out
    let (marketreq_tx, marketreq_rx): (Sender<MarketRequest>, Receiver<MarketRequest>) = mpsc::channel();

    // tx/rx for trades confirm
    let (reqconfirm_tx, reqconfirm_rx): (Sender<bool>, Receiver<bool>) = mpsc::channel();

    // tx/rx for user_data stream
    let (userdata_tx, userdata_rx): (Sender<ReceivedData>, Receiver<ReceivedData>) = mpsc::channel();

    // tx/rx for klines update line
    let (kline_tx, kline_rx): (Sender<ReceivedData>, Receiver<ReceivedData>) = mpsc::channel();
    let kline_tx2 = kline_tx.clone();
    let kline_tx3 = kline_tx.clone();

    // tx/rx for command lines
    let (cmd_tx, cmd_rx): (Sender<String>, Receiver<String>) = mpsc::channel();
    let cmd_tx_action = cmd_tx.clone();

    // thread(s) to pull live webstream data from binance
    let klines_thread = thread::Builder::new().name("klines_data_thread".to_string()).spawn (move || {
        binance_interface::live_binance_stream("ethusdt@kline_1m", &kline_tx, &init_tx, binance_structs::StreamType::KLine);
    });
    let klines_thread = thread::Builder::new().name("klines_data_thread".to_string()).spawn (move || {
        binance_interface::live_binance_stream("ltcusdt@kline_1m", &kline_tx2, &init_tx3, binance_structs::StreamType::KLine);
    });
    let klines_thread = thread::Builder::new().name("klines_data_thread".to_string()).spawn (move || {
        binance_interface::live_binance_stream("btcusdt@kline_1m", &kline_tx3, &init_tx4, binance_structs::StreamType::KLine);
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
                    println!("Printing API result from market order.");
                    println!("{}", result);
                    let filled_status: String = result["status"].as_str().unwrap().parse().unwrap();
                    if filled_status == "FILLED".to_string() {
                        reqconfirm_tx.send(true);
                    }
                    let log_str = format!("trading_result: {}", result.to_string());
                    logging_tx1.send(log_str);
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
        let mut ohlc_valid = vec![false; 3];

        // generate/initializeportfolio management variables
        let mut ohlc_history: Vec<Vec<Vec<f64>>> = Vec::new();
        let mut algo_status: Vec<i32> = vec![0];
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
        let mut p_data: Vec<Vec<Vec<f64>>> = vec![vec![Vec::new(); algo_status.len()]; ticker_list.len()];

        // ethereum specific things
        let eth_stepsize = 0.00001;
        let eth_min_notional = 10.0;

        // settings(numerical only)
        let mut settings = HashMap::new();
        settings.insert("ohlc_period", 60 * 1000);
        settings.insert("max_lookback_ms", settings["ohlc_period"] * 24 * 60);

        // generate stepsize and min_notional
        let exchange_info = binance_interface::binance_rest_api("exchange_info", epoch_ms(), "");
        let symbols_arr = exchange_info["symbols"].as_array().unwrap();
        for asset in symbols_arr {
            let symbol = asset["baseAsset"].as_str().unwrap().to_string();
            if symbols_interest.iter().position(|x| x == &symbol) != None {
                let index = symbols_interest.iter().position(|x| x == &symbol).unwrap();
                for filter in asset["filters"].as_array().unwrap() {
                    if filter["filterType"].as_str().unwrap() == "LOT_SIZE" {
                        stepsize[index] = filter["stepSize"].as_str().unwrap().parse().unwrap();
                    } else if filter["filterType"].as_str().unwrap() == "MIN_NOTIONAL" {
                        min_notional[index] = filter["minNotional"].as_str().unwrap().parse().unwrap();
                    }
                }
            }
        }

        // check if any values are unpopulated
        if stepsize.contains(&-1.0) {
            panic!("One or more elements of stepsize was not calculated.");
        }

        if min_notional.contains(&-1.0) {
            panic!("One or more elements of stepsize was not calculated.");
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
                                println!("amt_to_sell % eth_stepsize: {}", amt_to_sell % eth_stepsize);
                                amt_to_sell = amt_to_sell - (amt_to_sell % eth_stepsize);
                                println!("final amt_to_sell: {}", amt_to_sell);
                                println!("Attempting to sell {} amount of {}...", amt_to_sell, account_info["balances"][i]["asset"]);
                                let request = MarketRequest {
                                    symbol: symbol, 
                                    side: "SELL".to_string(), 
                                    timestamp: time_now,
                                    quantity: amt_to_sell,
                                    quoteOrderQty: -1.0,
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
                    
                    for (i, ticker) in ticker_list.iter().enumerate() {
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

                    logging_tx.send("predata: finished fetching predata.".to_string());
                    println!("finished with fetching predata.");
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
                    println!("\nprinting stepsize...");
                    for i in &stepsize {
                        print!("{} ", i);
                    }
                    println!("\nprinting min_notional...");
                    for i in &min_notional {
                        print!("{} ", i);
                    }
                    println!("\n done with printing variables.");
                }
            }

            // main trade/pm logic
            if running {
                // receive market data(trading only for now) and append to past list of trades
                let mut kline_iter = kline_rx.try_iter();
                let mut kline_valid = false;
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
                            if ohlc_valid[index] == true {
                                panic!("Valid value already in place in the next ohlc segment for symbol.");
                            }
                            let append_arr = vec![kline.open, kline.high, kline.low, kline.close, kline.quantity];
                            ohlc_history[index].push(append_arr);
                            ohlc_valid[index] = true;

                            // logging
                            let log_str = format!("new_ohlc: {} {} {} {} {} {}", kline.symbol, kline.open, kline.high, kline.low, kline.close, kline.quantity);
                            logging_tx.send(log_str);

                            // check if all symbol ohlcs are closed, and take action if they are
                            if !(ohlc_valid.contains(&false)) {
                                kline_valid = true;
                                ohlc_valid = vec![false; ticker_list.len()];
                            }
                        }
                    } else {
                        break;
                    }
                }
                
                if kline_valid {
                    println!("kline is valid. running trading logic.");

                    // slice to relevant part
                    let limit_len = (settings["max_lookback_ms"] / settings["ohlc_period"]) as usize;

                    // iterate through all the symbols
                    for ticker_i in 0..ticker_list.len() {
                        println!("Now on ticker: {}", ticker_list[ticker_i]);
                        if ohlc_history[ticker_i].len() >= limit_len {
                            ohlc_history[ticker_i] = ohlc_history[ticker_i][ohlc_history[ticker_i].len()-limit_len..].to_vec();
                            
                            // call master strategy
                            // keep in mind, signals returned direct from the function is either 0 or 1. This is different from algo_status, where
                            // the numbers denote which currency the algo is playing.
                            let (signals, new_p_data) = trading_strategies::master_strategy(&ohlc_history[ticker_i], &p_data[ticker_i]);
                            p_data[ticker_i] = new_p_data;
    
                            // logging real quick
                            logging_tx.send(format!("update: on ticker {}", ticker_list[ticker_i]));
                            
                            for (i, signal) in signals.iter().enumerate() {
                                println!("Current algo play is {}. ", algo_status[i]);
                                println!("Algorithm returned {} indicator.", signal);
                                if (signal != &algo_status[i]) && (signal == &0 || &algo_status[i] == &0) {
                                    println!("signal contradicts status, taking action.");

                                    // empty rx for trade confirm so only data in pipe is from the request we're about to send. 
                                    loop {
                                        let status = reqconfirm_rx.try_iter().next();
                                        if status.is_none() {
                                            break;
                                        }
                                    }

                                    // fetch account information and calculate relative split to put into play
                                    let mut balances = vec![-1.0; symbols_interest.len()];
                                    // calculate balance for each symbol in symbols_interest
                                    let account_info = binance_interface::binance_rest_api("get_accountinfo", time_now, "");
                                    logging_tx.send(format!("account_update: {}", account_info.to_string()));

                                    // calculate balances
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
    
                                    if balances[algo_status[i] as usize] == -1.0 {
                                        panic!("Invalid balance.");
                                    }

                                    let mut balances_log = "".to_string();
                                    for element in &balances {
                                        balances_log = format!("{} {}", balances_log, element);
                                    }
                                    logging_tx.send(format!("calculated balance: {}", balances_log));
    
                                    // if signal is 0(back to USDT), calculate amount to sell.
                                    // if signal is positive(into a currency), calculate USDT amount then multiply by price
                                    let mut amt = relative_split * balances[algo_status[i] as usize];
    
                                    // amt processing
                                    if signal != &0 {
                                        if amt <= min_notional[ticker_i] {
                                            break;
                                        }
                                    } else {
                                        amt -= amt % stepsize[ticker_i];
                                    }
                                    println!("final amt: {}", amt);
    
                                    if signal != &0 {
                                        let request_time = epoch_ms();
                                        let request = MarketRequest {
                                            symbol: ticker_list[ticker_i].to_string(), 
                                            side: "BUY".to_string(), 
                                            timestamp: request_time,
                                            quantity: -1.0,
                                            quoteOrderQty: amt, 
                                        };
                                        logging_tx.send(format!("requesting trade: {}", request.clone().to_string()));
                                        marketreq_tx.send(request.clone()).unwrap();
                                        algo_status[i] = (ticker_i + 1) as i32;
                                    } else {
                                        let request_time = epoch_ms();
                                        let request = MarketRequest {
                                            symbol: ticker_list[ticker_i].to_string(), 
                                            side: "SELL".to_string(), 
                                            timestamp: epoch_ms(),
                                            quantity: amt,
                                            quoteOrderQty: -1.0,
                                        };
                                        logging_tx.send(format!("requesting trade: {}", request.clone().to_string()));
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
