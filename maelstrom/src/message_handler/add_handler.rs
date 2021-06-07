use json::{object, JsonValue};

use crate::{error::MaelstromError, states::node_state::NodeState};

use super::MessageHandler;

pub struct AddHandler {}

impl MessageHandler for AddHandler {
    fn make_response_body(
        &self,
        message: &JsonValue,
        curr_state: &NodeState,
    ) -> Result<JsonValue, MaelstromError> {
        let delta = message["body"]["delta"].as_i32().unwrap();
        curr_state.new_message(delta);
        Ok(object! {type: "add_ok"})
    }
}
