use json::{object, JsonValue};

use crate::{error::MaelstromError, states::node_state::NodeState};

use super::MessageHandler;

pub struct ReadHandler {}

impl MessageHandler for ReadHandler {
    fn make_response_body(
        &self,
        _message: &JsonValue,
        curr_state: &NodeState,
    ) -> Result<JsonValue, MaelstromError> {
        let curr_value = curr_state.read_counters();
        Ok(object!(type: "read_ok", value: JsonValue::from(curr_value)))
    }
}
