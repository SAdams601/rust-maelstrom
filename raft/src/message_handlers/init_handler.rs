use json::{object, JsonValue};
use shared_lib::{error::MaelstromError, message_handler::MessageHandler, node_state::NodeState, message_utils::get_body};
use crate::raft_node_state::RaftNodeState;

struct InitHandler {}

impl MessageHandler for InitHandler {
    type State = RaftNodeState;

    fn make_response_body(
        &self,
        message: &json::JsonValue,
        curr_state: &Self::State,
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
