use json::{object, JsonValue};

use crate::{error::MaelstromError, states::node_state::NodeState};

use super::MessageHandler;

pub struct EchoHandler {}

impl MessageHandler for EchoHandler {
    fn make_response_body(
        &self,
        message: &JsonValue,
        _curr_state: &NodeState,
    ) -> Result<JsonValue, MaelstromError> {
        Ok(object! {type: "echo_ok", echo: message["body"]["echo"].clone()})
    }
}
