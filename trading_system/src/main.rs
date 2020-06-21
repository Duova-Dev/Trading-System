mod binance_interface;
mod trading_strategies;

use crate::binance_interface::interface;
use crate::trading_strategies::strategies;
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use std::thread;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::HashMap;
use std::convert::TryFrom;

use serde_json::{Value};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    // tx/rx for init
    let (init_tx, init_rx): (Sender<bool>, Receiver<bool>) = mpsc::channel();
    let init_tx2 = init_tx.clone();

    // tx/rx for trade data line
    let (trade_tx, trade_rx): (Sender<Value>, Receiver<Value>) = mpsc::channel();

    // tx/rx for depth map update line
    let (depth_tx, depth_rx): (Sender<Value>, Receiver<Value>) = mpsc::channel();

    // tx/rx for command lines
    let (cmd_tx, cmd_rx): (Sender<String>, Receiver<String>) = mpsc::channel();

    // tx/rx for new prompt
    let (prompt_tx, prompt_rx): (Sender<bool>, Receiver<bool>) = mpsc::channel(); 

    // thread to pull live webstream data from binance
    let _trades_thread = thread::spawn(move || {
        interface::live_binance_stream("btcusdt@trade", &trade_tx, &init_tx);
    });

    let _depth_thread = thread::spawn(move || {
        interface::live_binance_stream("btcusdt@depth@100ms", &depth_tx, &init_tx2);
    });

    // spawn action thread
    let _action_thread = thread::spawn(move || {
        let mut running = false;
        let mut settings = HashMap::new();
        settings.insert("max_lookback_ms", 5 * 60 * 1000);

        // main loop
        loop {
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
                }
            }

            // main trade logic
            if running {
                // system time 
                let now_instant = SystemTime::now();
                let epoch_now = now_instant
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards.");
                let time_now = u64::try_from(epoch_now.as_millis()).unwrap();

                // portfolio management
                // hashmap of algo id : capital allocation
            let mut capital_allocation = HashMap::new();
                capital_allocation.insert(1, 0.5);
                capital_allocation.insert(2, 0.5);

                // receive market data(trading only for now)
                let mut trade_data_iter = trade_rx.try_iter();
                let mut trades_in_window = Vec::new();
                loop {
                    let next_data = trade_data_iter.next();
                    if next_data != None {
                        let trade = next_data.unwrap();
                        trades_in_window.push(trade);
                    } else {
                        break;
                    }
                }

                // call algo if new trades received
                let new_trades_received = trades_in_window.len() > 0;
                if new_trades_received {
                    // remove outdated trades
                    let mut i = 0;
                    let mut max_i = trades_in_window.len() - 1;
                    while i <= max_i {
                        if trades_in_window[i]["E"].as_u64().unwrap() < time_now - settings["max_lookback_ms"] {
                            trades_in_window.remove(i);
                            max_i -= 1;
                        }
                        i += 1;
                    }

                    // feed trade data to algorithm
                    strategies::master_strategy(vec![1], trades_in_window, time_now);
                }

            }

            // send all clear to prompt thread
            prompt_tx.send(true).unwrap();
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

        if command == "quit" {
            break;
        }
        
        // send command to action thread
        cmd_tx.send(command.to_string()).unwrap();

        // continue with the all-clear from the action thread
        loop {
            let received = match prompt_rx.recv() {
                Ok(data) => data,
                Err(e) => {
                    false
                }
            };
            if received {
                break;
            }
        }
    }
    Ok(())
}
