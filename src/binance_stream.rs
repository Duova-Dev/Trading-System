// interfacing code for streamr



pub mod interface {
    use url::Url;
    use tungstenite::{connect, Message};
    use std::sync::mpsc;
    use std::sync::mpsc::{Sender, Receiver};
    use std::thread;

    pub fn live_trade_stream(stream_name: &str, tx: &Sender<String>) {

        let binance_base_endpoint = "wss://stream.binance.com:9443";
        
        let access_url = format!("{}/ws/{}", binance_base_endpoint, stream_name);

        let (mut socket, response) = 
            connect(Url::parse(&access_url).unwrap()).expect("Can't connect.");
        
        println!("Connected to the server");
        println!("Response HTTP code: {}", response.status());
        println!("Response contains the following headers:");

        for (ref header, _value) in response.headers() {
            println!("* {}", header);
        }

        socket
            .write_message(Message::Text(r#"
            {
                "method": "SUBSCRIBE",
                "params": [
                  "btcusdt@trade"
                ],
                "id": 1
              }
            "#.into()))
            .unwrap();
        loop {
            let msg = socket.read_message().expect("Error reading message");
            let msg_string = format!("{}", msg);
            //println!("Received: {}", msg_string);
            tx.send(msg_string).unwrap();
        }
    }
}
