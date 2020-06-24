// interfacing code for streamr

pub mod interface {
    use curl::easy::Easy;
    use std::io::{Write};
    use std::sync::mpsc::{Sender};
    use tungstenite::connect;
    use url::Url;
    use std::str;
    use std::fs::File;
    use std::io::Read;
    use serde_json::{Value, json, Error};
    use std::collections::HashMap;
    use sha2::Sha256;
    use hmac::{Hmac, Mac, NewMac};

    fn sign_hmac256(key: &str, message: &str) -> String {
        // returns hmac256 signature with hex
        type HmacSha256 = Hmac<Sha256>;
        let mut mac = HmacSha256::new_varkey(key.as_bytes())
            .expect("error generating sha256 key");
        mac.update(message.as_bytes());
        let signature_bytes_arr = mac.finalize().into_bytes();
        let signature_bytes = signature_bytes_arr.as_slice();
        let mut signature_str = String::new();
        for byte in signature_bytes {
            signature_str = format!("{}{:x}", signature_str, byte);
        }
        return signature_str;
    }

    fn rest_get(api_key: String, url: String) -> String { 
        let client = reqwest::blocking::Client::new();
        println!("API_KEY: {}", api_key);
        let response_str = client.get(&url)
            .header("X-MBX-APIKEY", &api_key)
            .send().unwrap()
            .text().unwrap();
        println!("response_str: {}", response_str);
        return "".to_string();
    }

    pub fn binance_rest_api(interface: &str, timestamp: u64){
        let base_url = "https://api.binance.com";
        let mut return_data = String::new();

        // find keys
        let mut key_file = File::open("../v0_1_0.key").unwrap();
        let mut contents = String::new();
        key_file.read_to_string(&mut contents);
        let keys_json: Value = serde_json::from_str(&contents).unwrap();
        let api_key = keys_json["api_key"].as_str().unwrap();
        let secret_key = keys_json["secret_key"].as_str().unwrap();

        // types of requests
        if interface == "new_listenkey" {
            let endpoint = "/api/v3/userDataStream";
            let post_url = format!("{}{}", base_url, endpoint);
            let client = reqwest::blocking::Client::new();
            let response: Value = client.post(&post_url)
                .body("{}")
                .header("X-MBX-APIKEY", api_key)
                .send().unwrap()
                .json().unwrap();
            println!("response: {}", response);
        } else if interface == "get_accountinfo" {
            let endpoint = "/api/v3/account";
            let mut final_url = format!("{}{}?", base_url, endpoint);
            let message = &format!("timestamp={}", timestamp);
            let generated_hmac = sign_hmac256(secret_key, message);
            final_url = format!("{}{}&signature={}", final_url, message, generated_hmac);
            rest_get(api_key.to_string(), final_url);
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
