use json::{object, JsonValue};

use super::MessageHandler;
use crate::node::NodeState;

pub struct ReadHandler {}

impl MessageHandler for ReadHandler {
    fn make_response_body(&self, _message: &JsonValue, curr_state: &NodeState) -> JsonValue {
        let messages: Vec<JsonValue> = curr_state
            .read_messages()
            .iter()
            .map(|i| JsonValue::from(*i))
            .collect();
        object!(type: "read_ok", messages: messages)
    }
}
