use json::object;

use crate::message_utils::get_body;

use super::MessageHandler;

pub struct InitHandler {}

impl MessageHandler for InitHandler {
    fn make_response_body(
        &self,
        message: &json::JsonValue,
        curr_state: &crate::node::NodeState,
    ) -> json::JsonValue {
        let body = get_body(message);
        curr_state.set_node_id(body["node_id"].to_string());
        object! {type: "init_ok"}
    }
}
