use json::{object, JsonValue};

use super::MessageHandler;
use crate::node::NodeState;

pub struct ReadHandler {}

impl MessageHandler for ReadHandler {
    fn make_response_body(&self, _message: &JsonValue, curr_state: &NodeState) -> JsonValue {
        let curr_value = curr_state.read_counters();
        object!(type: "read_ok", value: JsonValue::from(curr_value))
    }
}
