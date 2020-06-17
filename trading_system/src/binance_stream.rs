// interfacing code for streamr

pub mod interface {
    use curl::easy::Easy;
    use std::io::{Write};
    use std::sync::mpsc::{Sender};
    use tungstenite::connect;
    use url::Url;
    use std::str;
    use std::fs::File;

    pub fn live_binance_stream(stream_name: &str, tx: &Sender<String>) {
        let binance_base_endpoint = "wss://stream.binance.com:9443";

        let access_url = format!("{}/ws/{}", binance_base_endpoint, stream_name);

        let (mut socket, response) =
            connect(Url::parse(&access_url).unwrap()).expect("Can't connect.");

        println!("Connected to the server");
        println!("Response HTTP code: {}", response.status());
        println!("Response contains the following headers:");

        for (ref header, _value) in response.headers() {
            println!("* {}", header);
            println!("{}", header);
        }
        /*
        socket
            .write_message(Message::Text(
                r#"
            {
                "method": "SUBSCRIBE",
                "params": [
                  "btcusdt@trade"
                ],
                "id": 1
              }
            "#
                .into(),
            ))
            .unwrap();
        */

        tx.send("initialized".to_string()).unwrap();
        
        loop {
            let msg = socket.read_message().expect("Error reading message");
            let msg_string = format!("{}", msg);
            //println!("Received: {}", msg_string);
            tx.send(msg_string).unwrap();
        }
    }
    pub fn get_depth_snapshot(file_to_write: &str) -> std::io::Result<()> {
        //let mut buf = Vec::new();
        let mut buffer = File::create(file_to_write)?;
        let mut handle = Easy::new();
        handle.url("https://www.binance.com/api/v1/depth?symbol=BNBBTC&limit=1000").unwrap();

        let mut transfer = handle.transfer();
        transfer
            .write_function(|data| {
                buffer.write(data).unwrap();
                Ok(data.len())
            })
            .unwrap();
        transfer.perform().unwrap();
        //drop(transfer);
        //let s = String::from_utf8(buf).expect("Found invalid UTF-8");
        //return serde_json::from_str(&s).unwrap();
        Ok(())
    }
}
