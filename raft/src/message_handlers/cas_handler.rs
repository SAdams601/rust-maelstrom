use shared_lib::message_handler::MessageHandler;
use crate::raft_node_state::RaftState;
use json::{JsonValue, object };
use shared_lib::error::{MaelstromError, DefiniteError};
use shared_lib::message_utils::get_body;

pub struct CasHandler {}

impl MessageHandler<RaftState> for CasHandler {
    fn make_response_body(&self, message: &JsonValue, curr_state: &RaftState) -> Result<JsonValue, MaelstromError> {
        let body = get_body(message);
        let cas_result = curr_state.cas_value(body["key"].as_i32().unwrap(), body["from"].as_i32().unwrap(), body["to"].as_i32().unwrap());
        match cas_result {
            Ok(()) => Ok(object! {type: "cas_ok"}),
            Err(definite_error) => Err(MaelstromError {
                in_reply_to: body["msg_id"].as_i32().unwrap(),
                error: definite_error
            })
        }
    }
}
