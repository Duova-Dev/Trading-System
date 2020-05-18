mod streamr;

use crate::streamr::interface;

fn main() {
    interface::test_tungstenite("wss://stream.binance.com:9443/ws/btcusdt@trade");
}
