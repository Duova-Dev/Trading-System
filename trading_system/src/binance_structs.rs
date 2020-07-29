use serde::{Serialize, Deserialize};
use serde_json::{Value};

// data structures

pub enum ReceivedData {
    Trade(OccuredTrade),
    KLine(KLineMinute),
    Value(Value)
}

impl ReceivedData {
    pub fn as_trade(self) -> OccuredTrade {
        if let ReceivedData::Trade(c) = self {
            c
        } else {
            panic!("ReceivedData could not be expressed as an OccuredTrade");
        }
    }

    pub fn as_kline(self) -> KLineMinute {
        if let ReceivedData::KLine(c) = self {
            c
        } else {
            panic!("ReceivedData could not be expressed as an KLine");
        }
    }

    pub fn as_value(self) -> Value {
        if let ReceivedData::Value(c) = self {
            c
        } else {
            panic!("ReceivedData could not be expressed as an Value");
        }
    }
}

#[derive(Clone)]
pub struct KLineMinute {
    pub start_time: u64, 
    pub end_time: u64, 
    pub symbol: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub quantity: f64,
    pub num_trades: u64,
    pub closed: bool,
}

pub struct OccuredTrade {
    pub event_type: String,
    pub event_time: u64,
    pub symbol: String,
    pub trade_id: u64,
    pub price: f64,
    pub quantity: f64,
    pub buyer_id: u64,
    pub seller_id: u64,
    pub trade_time: u64,
    pub buyermm: bool,
    pub ignore: bool
}

pub enum StreamType {
    Trade, 
    Depth,
    KLine,
    UserData
}

#[derive(Clone)]
pub struct MarketRequest {
    pub symbol: String,
    pub side: String,
    pub timestamp: u64,
    pub quantity: f64,
}

impl MarketRequest {
    pub fn to_string(self) -> String {
        return format!("symbol={}&side={}&timestamp={}&quantity={}&type=MARKET", self.symbol, self.side, self.timestamp, self.quantity);
    }
}


// helper functions
pub fn deserialize_kline(raw_kline: Value) -> KLineMinute {
    KLineMinute {
        start_time: raw_kline["k"]["t"].as_u64().unwrap(), 
        end_time: raw_kline["k"]["T"].as_u64().unwrap(), 
        symbol: raw_kline["k"]["s"].to_string(),
        open: raw_kline["k"]["o"].as_str().unwrap().parse().unwrap(),
        high: raw_kline["k"]["h"].as_str().unwrap().parse().unwrap(),
        low: raw_kline["k"]["l"].as_str().unwrap().parse().unwrap(),
        close: raw_kline["k"]["c"].as_str().unwrap().parse().unwrap(),
        quantity: raw_kline["k"]["v"].as_str().unwrap().parse().unwrap(),
        num_trades: raw_kline["k"]["n"].as_u64().unwrap(),
        closed: raw_kline["k"]["x"].as_bool().unwrap()
    }
}

pub fn deserialize_trade(received_trade: Value) -> OccuredTrade {
    // manual deserialization because serde's derive has incompatible dependencies
    return OccuredTrade {
        event_type: received_trade["e"].to_string(),
        event_time: received_trade["E"].as_u64().unwrap(),
        symbol: received_trade["s"].to_string(),
        trade_id: received_trade["t"].as_u64().unwrap(),
        price: received_trade["p"].as_str().unwrap().parse().unwrap(),
        quantity: received_trade["q"].as_str().unwrap().parse().unwrap(),
        buyer_id: received_trade["b"].as_u64().unwrap(),
        seller_id: received_trade["a"].as_u64().unwrap(),
        trade_time: received_trade["T"].as_u64().unwrap(),
        buyermm: received_trade["m"].as_bool().unwrap(),
        ignore: received_trade["M"].as_bool().unwrap(),
    };
}
