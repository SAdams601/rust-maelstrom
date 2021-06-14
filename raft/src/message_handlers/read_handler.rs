use shared_lib::message_handler::MessageHandler;
use crate::raft_node_state::RaftState;
use json::{JsonValue, object};
use shared_lib::error::{MaelstromError, DefiniteError};
use shared_lib::message_utils::get_body;

pub struct ReadHandler {}

impl MessageHandler<RaftState> for ReadHandler {
    fn make_response_body(&self, message: &JsonValue, curr_state: &RaftState) -> Result<JsonValue, MaelstromError> {
        let body = get_body(message);
        let read_result = curr_state.read_value(body["key"].as_i32().unwrap());
        match read_result {
            Ok(read_value) => {
                Ok(object!{type: "read_ok", value: read_value.clone()})
            }
            Err(definite_error) => {
                Err(MaelstromError {
                    error: definite_error,
                    in_reply_to: body["msg_id"].as_i32().unwrap()
                })
            }
        }
    }
}
