mod binance_stream;

use crate::binance_stream::interface;

use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use std::thread;
use std::time::Duration;
use json;

fn main() {
    let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();

    // thread to pull live trade data from binance
    let trades_thread = thread::spawn(move || {
        interface::live_trade_stream("btcusdt@trade", &tx);
    });

    // loop to constantly process trading data
    for received in rx {
        let parsed = json::parse(&received).unwrap();
        println!("{}", parsed["p"]);
    }
    
    trades_thread.join().unwrap();

}
