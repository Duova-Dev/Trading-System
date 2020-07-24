mod binance_interface;
mod trading_strategies;
mod binance_structs;

use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use std::thread;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::HashMap;
use std::convert::TryFrom;
use serde::{Deserialize};
use serde_json::{Value};
use binance_structs::{ReceivedData, OccuredTrade};


fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    // tx/rx for init
    let (init_tx, init_rx): (Sender<bool>, Receiver<bool>) = mpsc::channel();
    let init_tx2 = init_tx.clone();

    // tx/rx for quit
    let (quit_tx, quit_rx): (Sender<bool>, Receiver<bool>) = mpsc::channel();

    // tx/rx for things to print
    let (print_tx, print_rx): (Sender<String>, Receiver<String>) = mpsc::channel();

    // tx/rx for trade data line
    let (trade_tx, trade_rx): (Sender<ReceivedData>, Receiver<ReceivedData>) = mpsc::channel();

    // tx/rx for depth map update line
    let (depth_tx, depth_rx): (Sender<ReceivedData>, Receiver<ReceivedData>) = mpsc::channel();

    // tx/rx for command lines
    let (cmd_tx, cmd_rx): (Sender<String>, Receiver<String>) = mpsc::channel();

    // thread to pull live webstream data from binance
    let _trades_thread = thread::Builder::new().name("trade_data_thread".to_string()).spawn (move || {
        binance_interface::live_binance_stream("btcusdt@trade", &trade_tx, &init_tx, binance_structs::StreamType::Trade);
    });

    let _depth_thread = thread::Builder::new().name("depth_data_thread".to_string()).spawn (move || {
        binance_interface::live_binance_stream("btcusdt@depth@100ms", &depth_tx, &init_tx2, binance_structs::StreamType::Depth);
    });

    // spawn action thread
    let _action_thread = thread::Builder::new().name("action_data_thread".to_string()).spawn(move || {
        // variables initialization
        let mut running = false;
        let mut settings = HashMap::new();
        // period is 1 minute for now
        settings.insert("ohlc_period", 60 * 1000);
        settings.insert("max_lookback_ms", settings["ohlc_period"] * 24 * 60);
        let mut trades_in_window: Vec<OccuredTrade> = Vec::new();
        let mut ohlc_history: Vec<Vec<f64>> = Vec::new();

        // time init
        let now_instant = SystemTime::now();
        let epoch_now = now_instant
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards.");
        let time_threshold = u64::try_from(epoch_now.as_millis()).unwrap() + settings["ohlc_period"];

        // main loop
        loop {
            // system time 
            let now_instant = SystemTime::now();
            let epoch_now = now_instant
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards.");
            let time_now = u64::try_from(epoch_now.as_millis()).unwrap();

            // check if command command
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
                }
            }

            // main trade/pm logic
            if running {
                // receive market data(trading only for now) and append to past list of trades
                let mut trade_data_iter = trade_rx.try_iter();
                let mut new_trades : Vec<OccuredTrade> = Vec::new();
                loop {
                    let next_data = trade_data_iter.next();
                    if !next_data.is_none() {
                        let trade = next_data.unwrap();
                        let trade_to_push = trade.as_trade();
                        new_trades.push(trade_to_push);
                    } else {
                        break;
                    }
                }
                trades_in_window.append(&mut new_trades);

                if time_now >= time_threshold {
                    // find ohlc of time period that just finished
                    let mut close: f64 = 0.0;
                    let mut open: f64 = 0.0;
                    let mut high = f64::MIN;
                    let mut low = f64::MAX;
                    let mut volume: f64 = 0.0;
                    for (i, trade) in trades_in_window.iter().enumerate() {
                        if i == 0 {
                            close = trade.price;
                            high = trade.price;
                            low = trade.price;
                        } else if i == trades_in_window.len() - 1 {
                            open = trade.price;
                        } 
                        if trade.price > high {
                            high = trade.price;
                        }
                        if trade.price < low {
                            low = trade.price;
                        }
                        volume += trade.quantity;
                    }

                    // append to ohlc_history 
                    let append_arr = vec![open, high, low, close, volume];
                    ohlc_history.push(append_arr);

                    // slice to relevant part
                    let limit_len = (settings["max_lookback_ms"] / settings["ohlc_period"]) as usize;
                    if ohlc_history.len() > limit_len {
                        ohlc_history = ohlc_history[ohlc_history.len()-limit_len..].to_vec();
                    }

                    // portfolio management
                    // hashmap of algo id : capital allocation
                    let mut capital_allocation = HashMap::new();
                    capital_allocation.insert(0, 1);
                    
                    // call master strategy
                    let signals = trading_strategies::master_strategy(&ohlc_history);


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
        if counter >= 2 {
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
