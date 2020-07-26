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
use binance_structs::{ReceivedData, OccuredTrade};
use helpers::{epoch_ms};


fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    // tx/rx for init
    let (init_tx, init_rx): (Sender<bool>, Receiver<bool>) = mpsc::channel();
    let init_tx2 = init_tx.clone();
    let init_tx3 = init_tx.clone();

    // tx/rx for quit
    let (quit_tx, quit_rx): (Sender<bool>, Receiver<bool>) = mpsc::channel();

    // tx/rx for things to print
    let (print_tx, print_rx): (Sender<String>, Receiver<String>) = mpsc::channel();

    // tx/rx for trades out
    let (marketreq_tx, marketreq_rx): (Sender<binance_structs::MarketRequest>, Receiver<binance_structs::MarketRequest>) = mpsc::channel();

    // tx/rx for user_data stream
    let (userdata_tx, userdata_rx): (Sender<ReceivedData>, Receiver<ReceivedData>) = mpsc::channel();

    // tx/rx for klines update line
    let (kline_tx, kline_rx): (Sender<ReceivedData>, Receiver<ReceivedData>) = mpsc::channel();

    // tx/rx for command lines
    let (cmd_tx, cmd_rx): (Sender<String>, Receiver<String>) = mpsc::channel();

    // thread to pull live webstream data from binance
    let _klines_thread = thread::Builder::new().name("klines_data_thread".to_string()).spawn (move || {
        binance_interface::live_binance_stream("ethusdt@kline_1m", &kline_tx, &init_tx2, binance_structs::StreamType::KLine);
    });
    
    // thread for user data
    let _user_data_thread = thread::Builder::new().name("user_data_thread".to_string()).spawn (move || {
        let listen_key = binance_interface::new_listenkey(epoch_ms());
        binance_interface::live_binance_stream(&listen_key, &userdata_tx, &init_tx3, binance_structs::StreamType::UserData);
    });
    
    // spawn action thread
    let _action_thread = thread::Builder::new().name("action_data_thread".to_string()).spawn(move || {
        // variables initialization
        let mut running = false;
        let mut diagnostic = false;
        let mut settings = HashMap::new();

        // period is 1 minute for now
        settings.insert("ohlc_period", 60 * 1000);
        settings.insert("max_lookback_ms", settings["ohlc_period"] * 24 * 60);
        let mut ohlc_history: Vec<Vec<f64>> = Vec::new();

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
                } else if command == "newlistenkey" {
                    binance_interface::binance_rest_api("new_listenkey", time_now);
                } else if command == "displayaccountinfo" {
                    println!("{}", binance_interface::binance_rest_api("get_accountinfo", time_now).to_string());
                } else if command == "testping" {
                    binance_interface::binance_rest_api("test_ping", time_now);
                } else if command == "testtime" {
                    binance_interface::binance_rest_api("test_time", time_now);
                } else if command == "exchangeinfo" {
                    binance_interface::binance_rest_api("exchange_info", time_now);
                } else if command == "selltousdt" {
                    // check account info and sells every asset to USDT
                    let account_info = binance_interface::binance_rest_api("get_accountinfo", time_now);
                    println!("account_info: {}", account_info);
                    
                    let mut i = 0;
                    loop {
                        if account_info["balances"][i].is_null() {
                            break;
                        } else {
                            println!("{}", account_info["balances"][i]);
                            if account_info["balances"][i]["asset"] != "USDT" && account_info["balances"][i]["free"] != 0 {
                                println!("Attempting to sell {} amount of {}...", account_info["balances"][i]["free"], account_info["balances"][i]["asset"]);
                                let request = binance_structs::MarketRequest {
                                    symbol: account_info["balances"][i]["asset"].to_string(), 
                                    side: "SELL".to_string(), 
                                    timestamp: time_now,
                                    quantity: account_info["balances"][i]["free"].as_str().unwrap().parse().unwrap()
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
                    
                } else if command == "testtrade" {
                    let request = binance_structs::MarketRequest {
                        symbol: "ETHUSDT".to_string(), 
                        side: "BUY".to_string(), 
                        timestamp: time_now,
                        quantity: 0.04
                    };
                    let response = binance_interface::binance_trade_api(request);
                    println!("response: {}", response);
                } else if command == "startdiagnostic" {
                    diagnostic = true;
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
                            kline_valid = true;
                            // append to ohlc_history 
                            let append_arr = vec![kline.open, kline.high, kline.low, kline.close, kline.quantity];
                            ohlc_history.push(append_arr);
                        }
                    } else {
                        break;
                    }
                }
                
                if kline_valid {
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

                    }
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
        if counter >= 1 {
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
