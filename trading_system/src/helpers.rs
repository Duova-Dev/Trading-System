use std::convert::TryFrom;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn epoch_ms() -> u64 {
    let now_instant = SystemTime::now();
    let epoch_now = now_instant
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards.");
    let time_now = u64::try_from(epoch_now.as_millis()).unwrap();
    return time_now;
}
