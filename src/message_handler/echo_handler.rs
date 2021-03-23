use json::{object, JsonValue};

use crate::node::NodeState;

use super::MessageHandler;

pub struct EchoHandler {}

impl MessageHandler for EchoHandler {
    fn make_response_body(&self, message: &JsonValue, _curr_state: &NodeState) -> JsonValue {
        object! {type: "echo_ok", echo: message["body"]["echo"].clone()}
    }
}
