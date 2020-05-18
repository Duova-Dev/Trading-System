mod streamr;

use crate::streamr::interface;

fn main() {
    println!("Hello, world!");
    interface::test_tungstenite("wss://stream.binance.com:9443/ws/btcusdt@trade");
}
