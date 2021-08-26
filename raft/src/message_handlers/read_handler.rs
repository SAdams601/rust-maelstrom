use shared_lib::message_handler::MessageHandler;
use crate::raft_node_state::RaftState;
use json::{JsonValue, object};
use shared_lib::error::{MaelstromError, DefiniteError, temporarily_unavailable};
use shared_lib::message_utils::{get_body, get_in_response_to};
use std::sync::Arc;
use crate::election_state::ElectionState;
use crate::election_state::State::LEADER;
use crate::log::Op::Read;

pub struct ReadHandler<'a> {
    election_state: Arc<ElectionState<'a>>
}

impl ReadHandler<'_> {
    pub fn init(election_state: Arc<ElectionState<'_>>) -> ReadHandler {
        ReadHandler {
            election_state
        }
    }
}

impl MessageHandler<RaftState> for ReadHandler<'_> {
    fn make_response_body(&self, message: &JsonValue, curr_state: &RaftState) -> Result<JsonValue, MaelstromError> {
        let body = get_body(message);
        if self.election_state.current_state() != LEADER {
            let error = temporarily_unavailable("Not a leader".to_string());
            let in_reply_to = body["msg_id"].as_i32().unwrap();
            return Err(MaelstromError {
                in_reply_to,
                error
            });
        }
        curr_state.append_single_entry(Read, self.election_state.current_term());
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
