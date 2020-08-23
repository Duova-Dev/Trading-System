
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
use crate::binance_structs;
use crate::helpers::epoch_ms;

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
        let byte_str = format!("{:x}", byte);
        if byte_str.len() < 2 {
            signature_str = format!("{}0{:x}", signature_str, byte);
        } else  {
            signature_str = format!("{}{:x}", signature_str, byte);
        }
    }
    return signature_str;
}

fn binance_rest_req(api_key: &str, url: String, req_type: String) -> String { 
    let client = reqwest::blocking::Client::new();
    let mut response_str = String::new();
    println!("requesting url: {}", url);
    if req_type == "get" {
        response_str = client.get(&url)
            .header("X-MBX-APIKEY", api_key)
            .send().unwrap()
            .text().unwrap();
    } else if req_type == "post" {
        response_str = client.post(&url)
            .header("X-MBX-APIKEY", api_key)
            .body("")
            .send().unwrap()
            .text().unwrap();
    }
    return response_str;
}

pub fn json_rest_req(url: String, req_type: String, msg: HashMap<&str, String>) -> String {
    let client = reqwest::blocking::Client::new();
    let mut response_str = String::new();
    println!("requesting url: {}", url);
    if req_type == "get" {
        let response = client.get(&url)
            .header("Content-Type","application/json")
            .json(&msg)
            .send().unwrap();
        let returned_headers = response.headers();
        response_str = response.text().unwrap();
    } else if req_type == "post" {
        let response = client.post(&url)
            .header("Content-Type","application/json")
            .json(&msg)
            .send().unwrap();
        let returned_headers = response.headers();
        response_str = response.text().unwrap();
    }
    return response_str;
}

pub fn binance_trade_api(request: binance_structs::MarketRequest) -> Value{
    // find keys
    let mut key_file = File::open("../v0_1_0.key").unwrap();
    let mut contents = String::new();
    key_file.read_to_string(&mut contents);
    let keys_json: Value = serde_json::from_str(&contents).unwrap();
    let api_key = keys_json["api_key"].as_str().unwrap();
    let secret_key = keys_json["secret_key"].as_str().unwrap();

    let endpoint = "https://api.binance.us/api/v3/order?";
    let message = request.to_string();
    let generated_hmac = sign_hmac256(secret_key, &message);
    let final_url = format!("{}{}&signature={}", endpoint, message, generated_hmac);
    let raw_response_str = binance_rest_req(api_key, final_url, "post".to_string());
    return serde_json::from_str(&raw_response_str).unwrap();
}

// wrapper functions for convenient access to certain api elements
pub fn new_listenkey(timestamp: u64) -> String {
    let listen_key = binance_rest_api("new_listenkey", timestamp, "");
    return listen_key["listenKey"].as_str().unwrap().to_string();
}

pub fn fetch_klines(symbol_str: &String, end_time: u64, lookback: u64) -> Vec<Vec<f64>> {
    let symbol: &str = symbol_str;
    let lookback_ms = lookback * 60 * 1000;
    let message = format!("symbol={}&interval=1m&startTime={}&endTime={}", symbol, end_time-lookback_ms, end_time);
    let mut vec_to_return: Vec<Vec<f64>> = Vec::new();
    // let message = format!("symbol={}&interval=1m", symbol);
    let klines_value = binance_rest_api("historicalkline", end_time, &message).as_array().unwrap().clone();
    for period in klines_value {
        let open : f64 = period[1].as_str().unwrap().parse().unwrap();
        let high : f64 = period[2].as_str().unwrap().parse().unwrap();
        let low : f64 = period[3].as_str().unwrap().parse().unwrap();
        let close : f64 = period[4].as_str().unwrap().parse().unwrap();
        let volume : f64 = period[5].as_str().unwrap().parse().unwrap();
        let vec_to_push = vec![open, high, low, close, volume];
        vec_to_return.push(vec_to_push);
    }
    return vec_to_return;
}

pub fn fetch_balances(symbols_interest: Vec<String>) -> Vec<f64> {
    /* 
        Input a list of tickers(e.g. ETHUSDT). Return vector will be same length, and display quantity of each crypto in balance.
    */
    // fetch account information and calculate relative split to put into play
    let mut balances = vec![-1.0; symbols_interest.len()];
    // calculate balance for each symbol in symbols_interest
    let account_info = binance_rest_api("get_accountinfo", epoch_ms(), "");
    let mut j = 0;
    loop {
        if account_info["balances"][j].is_null() {
            break;
        } else {
            let balance: f64 = account_info["balances"][j]["free"].as_str().unwrap().parse().unwrap();
            let balance_ticker: String = account_info["balances"][j]["asset"].as_str().unwrap().to_string();
            for k in 0..symbols_interest.len() {
                if balance_ticker == symbols_interest[k] {
                    balances[k] = balance;
                }
            }
        }
        j += 1;
    }
    return balances;
}

pub fn binance_rest_api(interface: &str, timestamp: u64, arguments: &str) -> Value {
    /*
        Requests certain things fron the Binance REST API based on predefined settings.
        This function returns the raw Serde Value, so if you want to parse the output, write a wrapper function.
    */
    let base_url = "https://api.binance.us";
    let mut return_data = String::new();

    // find keys
    let mut key_file = File::open("../v0_1_0.key").unwrap();
    let mut contents = String::new();
    key_file.read_to_string(&mut contents);
    let keys_json: Value = serde_json::from_str(&contents).unwrap();
    let api_key = keys_json["api_key"].as_str().unwrap();
    let secret_key = keys_json["secret_key"].as_str().unwrap();

    let mut final_url = String::new();
    let mut req_type = String::new();

    // types of requests
    if interface == "new_listenkey" {
        let endpoint = "/api/v3/userDataStream";
        final_url = format!("{}{}", base_url, endpoint);
        req_type = "post".to_string();
    } else if interface == "get_accountinfo" {
        println!("running get_accountinfo...");
        let endpoint = "/api/v3/account";
        let appended_endpoint = format!("{}{}?", base_url, endpoint);
        let message = &format!("timestamp={}&recvWindow=5000", timestamp);
        let generated_hmac = sign_hmac256(secret_key, message);
        final_url = format!("{}{}&signature={}", appended_endpoint, message, generated_hmac);
        req_type = "get".to_string();
    } else if interface == "test_ping" {
        let endpoint = "/api/v3/ping";
        final_url = format!("{}{}?", base_url, endpoint);
        req_type = "get".to_string();
    } else if interface == "test_time" {
        let endpoint = "/api/v3/time";
        final_url = format!("{}{}?", base_url, endpoint); 
        req_type = "get".to_string();
    } else if interface == "exchange_info" {
        let endpoint = "/api/v3/exchangeInfo";
        final_url = format!("{}{}?", base_url, endpoint); 
        req_type = "get".to_string();
    } else if interface == "historicalkline" {
        let endpoint = "/api/v3/klines"; 
        final_url = format!("{}{}?{}", base_url, endpoint, arguments);
        req_type = "get".to_string();
    } else if interface == "exchange_info" {
        let endpoint = "/api/v3/exchangeInfo";
        final_url = format!("{}{}?", base_url, endpoint); 
        req_type = "get".to_string();
    }

    let raw_response_str = binance_rest_req(api_key, final_url, req_type);
    // println!("raw_response_str: {}", raw_response_str);
    return serde_json::from_str(&raw_response_str).unwrap();
}

pub fn live_binance_stream(stream_name: &str, data_tx: &Sender<binance_structs::ReceivedData>, init_tx: &Sender<bool>, stream_type: binance_structs::StreamType) {
    let binance_base_endpoint = "wss://stream.binance.com:9443";

    let access_url = format!("{}/ws/{}", binance_base_endpoint, stream_name);

    println!("attempting to access: {}", access_url);

    let (mut socket, response) =
        connect(Url::parse(&access_url).unwrap()).expect("Can't connect.");

    // println!("Connected to the server");
    // println!("Response HTTP code: {}", response.status());
    // println!("Response contains the following headers:");

    for (ref header, _value) in response.headers() {
        // println!("* {}", header);
        // println!("{}", header);
    }

    init_tx.send(true).unwrap();
    drop(init_tx);

    loop {
        let msg = socket.read_message().expect("Error reading message");
        let msg_string = format!("{}", msg);
        if msg_string.chars().next().unwrap() != '{' {
            continue;
        }
        let parsed_msg: Value = serde_json::from_str(&msg_string).unwrap();
        match stream_type {
            binance_structs::StreamType::Trade => {
                let constructed_trade = binance_structs::deserialize_trade(parsed_msg);
                data_tx.send(binance_structs::ReceivedData::Trade(constructed_trade)).unwrap();
            }
            binance_structs::StreamType::Depth => {
                data_tx.send(binance_structs::ReceivedData::Value(parsed_msg)).unwrap();
            }
            binance_structs::StreamType::KLine => {
                let constructed_kline = binance_structs::deserialize_kline(parsed_msg);
                data_tx.send(binance_structs::ReceivedData::KLine(constructed_kline)).unwrap();
            } 
            binance_structs::StreamType::UserData => {
                data_tx.send(binance_structs::ReceivedData::Value(parsed_msg)).unwrap();
            } 
        }
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

