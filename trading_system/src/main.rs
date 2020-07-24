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
        let mut running = false;
        let mut settings = HashMap::new();
        settings.insert("max_lookback_ms", 5 * 60 * 1000);
        let mut trades_in_window: Vec<OccuredTrade> = Vec::new();

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
                }
            }

            // main trade/pm logic
            if running {
                // portfolio management
                // hashmap of algo id : capital allocation
                let mut capital_allocation = HashMap::new();
                capital_allocation.insert(0, 1);

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


                // call algo if new trades received
                let new_trades_received = new_trades.len() > 0;
                if new_trades_received {
                    // removed outdated trades
                    let mut i = 0;
                    let mut max_i = trades_in_window.len() - 1;
                    while i <= max_i {
                        if trades_in_window[i].trade_time < time_now - settings["max_lookback_ms"] {
                            trades_in_window.remove(i);
                            max_i -= 1;
                        } else {
                            i += 1;
                        }
                    }

                    // feed trade data to algorithm
                    trading_strategies::master_strategy(vec![1], &trades_in_window, time_now);
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
