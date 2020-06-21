// interfacing code for streamr

pub mod interface {
    use curl::easy::Easy;
    use std::io::{Write};
    use std::sync::mpsc::{Sender};
    use tungstenite::connect;
    use url::Url;
    use std::str;
    use std::fs::File;
    use serde_json::{Value, json, Error};
    use std::collections::HashMap;

    pub fn binance_rest_api(interface: &str) {
        let base_url = "https://api.binance.com";
        if interface == "listenkey_request" {
            let listenkey_endpoint = "/api/v3/userDataStream";

        }

    }

    pub fn live_binance_stream(stream_name: &str, data_tx: &Sender<Value>, init_tx: &Sender<bool>) {
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

        init_tx.send(true).unwrap();
        drop(init_tx);

        loop {
            let msg = socket.read_message().expect("Error reading message");
            let msg_string = format!("{}", msg);
            let parsed_msg: Value = serde_json::from_str(&msg_string).unwrap();
            //println!("Received: {}", msg_string);
            data_tx.send(parsed_msg).unwrap();
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
