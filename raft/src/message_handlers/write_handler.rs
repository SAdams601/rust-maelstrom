use shared_lib::message_handler::MessageHandler;
use crate::raft_node_state::RaftState;
use json::{ JsonValue, object };
use shared_lib::error::{MaelstromError, temporarily_unavailable};
use shared_lib::message_utils::{get_body, get_in_response_to};
use std::sync::Arc;
use crate::election_state::ElectionState;
use crate::election_state::State::LEADER;
use crate::log::Op::Write;
use crate::message_handlers::proxy_to_leader::proxy_request_to_leader;
use std::option::Option::Some;

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
        unreachable!()
    }

    fn get_response_body(&self, message: &JsonValue, curr_state: &RaftState) -> Result<Option<JsonValue>, MaelstromError> {
        let body = get_body(message);
        if self.election_state.current_state() != LEADER {
            let result = proxy_request_to_leader(body, curr_state, self.election_state.clone());
            return result.map(|jv| Some(jv));
        }
        let key = body["key"].as_i32().unwrap();
        let value = body["value"].as_i32().unwrap();
        let requester = message["src"].as_str().unwrap().to_string();
        let msg_id = body["msg_id"].as_i32().unwrap();
        curr_state.append_single_entry(Write{key, value, requester, msg_id}, self.election_state.current_term());
        Ok(None)
    }
}
