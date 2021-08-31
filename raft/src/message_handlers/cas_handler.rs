use shared_lib::message_handler::MessageHandler;
use crate::raft_node_state::RaftState;
use json::{JsonValue, object};
use shared_lib::error::{MaelstromError, DefiniteError, temporarily_unavailable};
use shared_lib::message_utils::{get_body, get_in_response_to};
use crate::election_state::ElectionState;
use std::sync::Arc;
use crate::election_state::State::LEADER;
use crate::log::Op::CAS;
use crate::message_handlers::proxy_to_leader::proxy_request_to_leader;
use std::option::Option::Some;

pub struct CasHandler<'a> {
    election_state: Arc<ElectionState<'a>>,
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
        unreachable!()
    }

    fn get_response_body(&self, message: &JsonValue, curr_state: &RaftState) -> Result<Option<JsonValue>, MaelstromError> {
        let body = get_body(message);
        if self.election_state.current_state() != LEADER {
            let result = proxy_request_to_leader(body, curr_state, self.election_state.clone());
            return result.map(|jv| Some(jv));
        }
        let key = body["key"].as_i32().unwrap();
        let from = body["from"].as_i32().unwrap();
        let to = body["to"].as_i32().unwrap();
        let requester = message["src"].as_str().unwrap().to_string();
        let msg_id = body["msg_id"].as_i32().unwrap();
        curr_state.append_single_entry(CAS { key, from, to, requester, msg_id }, self.election_state.current_term());
        Ok(None)
    }
}


pub fn check_cas_result(cas_result: Result<(), DefiniteError>, in_reply_to: i32) -> Result<JsonValue, MaelstromError> {
    match cas_result {
        Ok(()) => Ok(object! {type: "cas_ok", in_reply_to: in_reply_to}),
        Err(definite_error) => Err(MaelstromError {
            in_reply_to,
            error: definite_error,
        })
    }
}

