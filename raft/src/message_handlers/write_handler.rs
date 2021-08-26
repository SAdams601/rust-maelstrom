use shared_lib::message_handler::MessageHandler;
use crate::raft_node_state::RaftState;
use json::{ JsonValue, object };
use shared_lib::error::{MaelstromError, temporarily_unavailable};
use shared_lib::message_utils::{get_body, get_in_response_to};
use std::sync::Arc;
use crate::election_state::ElectionState;
use crate::election_state::State::LEADER;
use crate::log::Op::Write;

pub struct WriteHandler<'a> {
    election_state: Arc<ElectionState<'a>>
}

impl WriteHandler<'_> {
    pub fn init(election_state: Arc<ElectionState<'_>>) -> WriteHandler {
        WriteHandler {
            election_state
        }
    }
}

impl MessageHandler<RaftState> for WriteHandler<'_> {
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

        curr_state.append_single_entry(Write, self.election_state.current_term());
        curr_state.write_value(body["key"].as_i32().unwrap(), body["value"].as_i32().unwrap());
        Ok(object!(type: "write_ok"))
    }
}
