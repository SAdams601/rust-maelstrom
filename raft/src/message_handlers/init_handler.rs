use json::{object, JsonValue};
use shared_lib::{error::MaelstromError, message_handler::MessageHandler, node_state::NodeState, message_utils::get_body};
use crate::raft_node_state::RaftState;

pub struct InitHandler {}

impl InitHandler {

    pub fn init() -> InitHandler { InitHandler {} }

}

impl MessageHandler<RaftState> for InitHandler {

    fn make_response_body(
        &self,
        message: &json::JsonValue,
        curr_state: &RaftState,
    ) -> Result<JsonValue, MaelstromError> {
        let body = get_body(message);
        let node_id = body["node_id"].to_string();
        curr_state.set_node_id(node_id.clone());
        curr_state.init_log(node_id.clone());
        curr_state.set_other_node_ids(
            body["node_ids"]
                .members()
                .map(|jv| jv.as_str().unwrap())
                .collect(),
        );
        Ok(object! {type: "init_ok"})
    }
}
