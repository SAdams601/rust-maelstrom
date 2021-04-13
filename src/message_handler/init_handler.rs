use json::{object, JsonValue};

use crate::{error::MaelstromError, message_utils::get_body, states::node_state::NodeState};

use super::MessageHandler;

pub struct InitHandler {}

impl MessageHandler for InitHandler {
    fn make_response_body(
        &self,
        message: &json::JsonValue,
        curr_state: &NodeState,
    ) -> Result<JsonValue, MaelstromError> {
        let body = get_body(message);
        curr_state.set_node_id(body["node_id"].to_string());
        curr_state.set_other_node_ids(
            body["node_ids"]
                .members()
                .map(|jv| jv.as_str().unwrap())
                .collect(),
        );
        Ok(object! {type: "init_ok"})
    }
}
