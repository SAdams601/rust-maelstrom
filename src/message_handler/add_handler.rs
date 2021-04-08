use json::object;

use crate::states::node_state::NodeState;

use super::MessageHandler;

pub struct AddHandler {}

impl MessageHandler for AddHandler {
    fn make_response_body(
        &self,
        message: &json::JsonValue,
        curr_state: &NodeState,
    ) -> json::JsonValue {
        let delta = message["body"]["delta"].as_i32().unwrap();
        curr_state.new_message(delta);
        object! {type: "add_ok"}
    }
}
