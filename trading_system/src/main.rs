mod binance_stream;

use crate::binance_stream::interface;

use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use std::thread;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    // tx/rx for data lines
    let (data_tx, data_rx): (Sender<String>, Receiver<String>) = mpsc::channel();
    let data_tx2 = data_tx.clone();

    // tx/rx for command lines
    let (cmd_tx, cmd_rx): (Sender<String>, Receiver<String>) = mpsc::channel();

    // tx/rx for new prompt
    let (prompt_tx, prompt_rx): (Sender<bool>, Receiver<bool>) = mpsc::channel(); 

    // thread to pull live webstream data from binance
    let _trades_thread = thread::spawn(move || {
        interface::live_binance_stream("btcusdt@trade", &data_tx);
    });

    let _depth_thread = thread::spawn(move || {
        interface::live_binance_stream("btcusdt@depth@100ms", &data_tx2);
    });

    // spawn action thread
    let _action_thread = thread::spawn(move || {
        let running = false;

        // main loop
        loop {
            // receive command
            let mut command_good = true;
            let command = match cmd_rx.recv() {
                Ok(data) => data,
                Err(e) => {
                    command_good = false;
                    "error".to_string()
                }
            };
            if command_good {
                println!("From cmd_rx: {}", command);
            }

            // send all clear to prompt thread
            prompt_tx.send(true).unwrap();
        }

    });


    // check if initialization complete
    let mut counter = 0;
    loop {
        let received = data_rx.recv().unwrap();
        if received == "initialized" {
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
