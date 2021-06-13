use json::{object, JsonValue};
use shared_lib::{error::MaelstromError, message_handler::MessageHandler};
use crate::states::maelstrom_node_state::MaelstromState;

pub struct AddHandler {}

impl MessageHandler<MaelstromState> for AddHandler {

    fn make_response_body(
        &self,
        message: &JsonValue,
        curr_state: &MaelstromState,
    ) -> Result<JsonValue, MaelstromError> {
        let delta = message["body"]["delta"].as_i32().unwrap();
        curr_state.new_message(delta);
        Ok(object! {type: "add_ok"})
    }
}
