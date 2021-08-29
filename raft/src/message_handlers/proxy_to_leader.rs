use shared_lib::error::{DefiniteError, temporarily_unavailable, MaelstromError};
use std::sync::Arc;
use crate::election_state::ElectionState;
use json::JsonValue;
use crate::raft_node_state::RaftState;
use crate::election_state::State::LEADER;
use shared_lib::rpc::send_rpc;

pub fn proxy_request_to_leader(body: &JsonValue, curr_state: &RaftState, election_state: Arc<ElectionState>) -> Result<JsonValue, MaelstromError> {
    let maybe_leader = election_state.get_leader();
    if maybe_leader.is_none() {
        let in_reply_to = body["msg_id"].as_i32().unwrap();
        return Err(
            MaelstromError {
                in_reply_to,
                error: temporarily_unavailable("not a leader".to_string())
            });
    }
    let leader = maybe_leader.unwrap();
    let leader_response = send_rpc(curr_state, &mut body.clone(), &leader)["body"].clone();
    Ok(leader_response)
}