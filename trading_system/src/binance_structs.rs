
use serde::{Serialize, Deserialize};
use serde_json::{Value};

pub enum ReceivedData {
    Trade(OccuredTrade),
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

pub fn deserialize_trade(received_trade: Value) -> OccuredTrade {
    // manual deserialization because serde's derive has incompatible dependencies
    return OccuredTrade {
        event_type: received_trade["e"].to_string(),
        event_time: received_trade["E"].as_u64().unwrap(),
        symbol: received_trade["s"].to_string(),
        trade_id: received_trade["t"].as_u64().unwrap(),
        price: received_trade["p"].as_f64().unwrap(),
        quantity: received_trade["q"].as_f64().unwrap(),
        buyer_id: received_trade["b"].as_u64().unwrap(),
        seller_id: received_trade["a"].as_u64().unwrap(),
        trade_time: received_trade["T"].as_u64().unwrap(),
        buyermm: received_trade["m"].as_bool().unwrap(),
        ignore: received_trade["M"].as_bool().unwrap(),
    };
}

pub enum StreamType {
    Trade, 
    Depth
}

pub enum TradeOrder {
    Limit(LimitRequest),
    Market(MarketRequest),
}

pub struct LimitRequest {
    symbol: String,
    side: String,
    timestamp: u64,
    timeInForce: String,
    quantity: f64,
    price: u64,
}

pub struct MarketRequest {
    symbol: String,
    side: String,
    timestamp: u64,
    quantity: f64,
}

// below is not supported yet
struct StopLossRequest {
    symbol: String,
    side: String,
    timestamp: u64,
}

struct StopLossLimitRequest {
    symbol: String,
    side: String,
    timestamp: u64,
}

struct TakeProfitRequest {
    symbol: String,
    side: String,
    timestamp: u64,
}

struct TakeProfitLimitRequest {
    symbol: String,
    side: String,
    timestamp: u64,
}
