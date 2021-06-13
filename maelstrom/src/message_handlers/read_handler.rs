use json::{object, JsonValue};
use shared_lib::{error::MaelstromError, message_handler::MessageHandler};
use crate::states::maelstrom_node_state::MaelstromState;

pub struct ReadHandler {}

impl MessageHandler<MaelstromState> for ReadHandler {

    fn make_response_body(
        &self,
        _message: &JsonValue,
        curr_state: &MaelstromState,
    ) -> Result<JsonValue, MaelstromError> {
        let curr_value = curr_state.read_counters();
        Ok(object!(type: "read_ok", value: JsonValue::from(curr_value)))
    }
}
