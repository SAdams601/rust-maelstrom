use shared_lib::message_handler::MessageHandler;
use crate::raft_node_state::RaftState;
use json::{JsonValue, object };
use shared_lib::error::{MaelstromError, DefiniteError, temporarily_unavailable};
use shared_lib::message_utils::{get_body, get_in_response_to};
use crate::election_state::ElectionState;
use std::sync::Arc;
use crate::election_state::State::LEADER;
use crate::log::Op::CAS;
use crate::message_handlers::proxy_to_leader::proxy_request_to_leader;

pub struct CasHandler<'a> {
    election_state: Arc<ElectionState<'a>>
}

impl CasHandler<'_> {
    pub fn init(election_state: Arc<ElectionState<'_>>) -> CasHandler {
        CasHandler {
            election_state
        }
    }
}

impl MessageHandler<RaftState> for CasHandler<'_> {
    fn make_response_body(&self, message: &JsonValue, curr_state: &RaftState) -> Result<JsonValue, MaelstromError> {
        let body = get_body(message);
        if self.election_state.current_state() != LEADER {
            return proxy_request_to_leader(body, curr_state, self.election_state.clone());
        }
        curr_state.append_single_entry(CAS, self.election_state.current_term());
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
