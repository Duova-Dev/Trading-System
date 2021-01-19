use serde::{Serialize, Deserialize};
use serde_json::{Value};
use std::fmt;

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
    /*
        Struct for a market order. 
        quantity or quoteOrderQty must be -1.0. One and only one must be a valid value. 
    */
    pub symbol: String,
    pub side: String,
    pub timestamp: u64,
    pub quantity: f64, 
    pub quote_order_qty: f64,
}

impl MarketRequest {
    pub fn to_string(self) -> String {
        if self.quantity == -1.0 && self.quote_order_qty != -1.0 {
            return format!("symbol={}&side={}&timestamp={}&quoteOrderQty={:.8}&type=MARKET", self.symbol, self.side, self.timestamp, self.quote_order_qty);
        } else if self.quote_order_qty == -1.0 && self.quantity != -1.0 {
            return format!("symbol={}&side={}&timestamp={}&quantity={:.8}&type=MARKET", self.symbol, self.side, self.timestamp, self.quantity);
        } else {
            panic!("MarketRequest format is wrong. Quantity and QuoteOrderQty are both/neither -1.0.");
        }
    }
}

// helper functions
pub fn deserialize_kline(raw_kline: Value) -> KLineMinute {
    KLineMinute {
        start_time: raw_kline["k"]["t"].as_u64().unwrap(), 
        end_time: raw_kline["k"]["T"].as_u64().unwrap(), 
        symbol: raw_kline["k"]["s"].as_str().unwrap().parse().unwrap(),
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


// Error types
pub struct APIError;

// Implement std::fmt::Display for AppError
impl fmt::Display for APIError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Something happened while interacting with an external API.") // user-facing output
    }
}

// Implement std::fmt::Debug for AppError
impl fmt::Debug for APIError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Erorr while interfacing with external API: {{ file: {}, line: {} }}", file!(), line!()) // programmer-facing output
    }
}