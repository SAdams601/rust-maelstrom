use json::{object, JsonValue};
use shared_lib::{error::MaelstromError, message_handler::MessageHandler};
use crate::states::maelstrom_node_state::MaelstromNodeState;

pub struct AddHandler {}

impl MessageHandler for AddHandler {
    type State = MaelstromNodeState;

    fn make_response_body(
        &self,
        message: &JsonValue,
        curr_state: &MaelstromNodeState,
    ) -> Result<JsonValue, MaelstromError> {
        let delta = message["body"]["delta"].as_i32().unwrap();
        curr_state.new_message(delta);
        Ok(object! {type: "add_ok"})
    }
}
