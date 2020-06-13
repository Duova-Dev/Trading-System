mod binance_stream;

use crate::binance_stream::interface;

use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use std::thread;
use std::fs::File;
use std::io::prelude::*;
use serde_json;
use std::any::type_name;
use csv::Writer;


fn type_of<T>(_: T) -> &'static str {
    type_name::<T>()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();

    // thread to pull live trade data from binance
    let _trades_thread = thread::spawn(move || {
        interface::live_trade_stream("btcusdt@depth", &tx);
    });

    interface::get_depth_snapshot().unwrap();

    let mut wtr = Writer::from_path("data/10minutechunk.csv")?;
    wtr.write_record(&["timestamp", "price,", "quantity", "buyermm"])?;

    let mut _counter = 0;
    let mut delta_time: u64;
    let mut initial_time: u64 = 0;
    let mut first_run_occured = false;


    // loop to constantly process trading data
    for received in rx {
        let trade: serde_json::Value = serde_json::from_str(&received).unwrap();

        // check if timestamp is present
        if trade["E"].is_null() {
            continue;
        }
        
        if first_run_occured == false {
            println!("initial trade: {}", trade["E"]);
            initial_time = trade["E"].as_u64().unwrap();
            first_run_occured = true;
        }

        delta_time = trade["E"].as_u64().unwrap() - initial_time;

        // blue means buy order put out, red means sell order put out
        if trade["m"] == true {
            colour::red_ln!("{}", trade["p"]);
        } else {
            colour::blue_ln!("{}", trade["p"]);
        }

        wtr.write_record(&[
            trade["E"].to_string(), 
            trade["p"].to_string(),
            trade["q"].to_string(),
            trade["m"].to_string(),
        ])?;

        _counter += 1;

        println!("delta_time: {}", delta_time);
        
        if delta_time >= 10000 {
            break;
        }
    }
    _trades_thread.join().unwrap();
    wtr.flush()?;
    Ok(())
}
