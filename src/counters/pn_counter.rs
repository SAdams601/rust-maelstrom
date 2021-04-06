use json::{object, JsonValue};

use super::g_counter::GCounter;

pub struct PnCounter {
    inc: GCounter,
    dec: GCounter,
}

impl PnCounter {
    pub fn init() -> PnCounter {
        PnCounter {
            inc: GCounter::init(),
            dec: GCounter::init(),
        }
    }

    pub fn from_json(json: &JsonValue) -> PnCounter {
        PnCounter {
            inc: GCounter::from_json(&json["inc"]),
            dec: GCounter::from_json(&json["dec"]),
        }
    }

    pub fn to_json(&self) -> JsonValue {
        object! {inc: self.inc.to_json(), dec: self.dec.to_json()}
    }

    pub fn read(&self) -> i32 {
        self.inc.read() - self.dec.read()
    }

    pub fn merge(&mut self, other: PnCounter) {
        self.inc.merge(other.inc);
        self.dec.merge(other.dec);
    }

    pub fn add(&mut self, node_id: String, delta: i32) {
        if 0 <= delta {
            self.inc.add(node_id, delta);
        } else {
            self.dec.add(node_id, -delta);
        }
    }
}
