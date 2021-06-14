use shared_lib::message_handler::MessageHandler;
use crate::raft_node_state::RaftState;
use json::{ JsonValue, object };
use shared_lib::error::MaelstromError;
use shared_lib::message_utils::get_body;

pub struct WriteHandler {}

impl MessageHandler<RaftState> for WriteHandler {
    fn make_response_body(&self, message: &JsonValue, curr_state: &RaftState) -> Result<JsonValue, MaelstromError> {
        let body = get_body(message);
        curr_state.write_value(body["key"].as_i32().unwrap(), body["value"].as_i32().unwrap());
        Ok(object!(type: "write_ok"))
    }
}
