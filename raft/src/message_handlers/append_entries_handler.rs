use shared_lib::message_handler::MessageHandler;
use crate::raft_node_state::RaftState;
use json::{JsonValue, object};
use shared_lib::error::{MaelstromError, DefiniteError, abort};
use crate::election_state::{ElectionState, election_loop};
use std::sync::Arc;
use shared_lib::message_utils::{get_body, get_in_response_to};
use crate::log::{Entry, parse_entries};
use std::cmp::min;
use shared_lib::stdio::write_log;

pub struct AppendEntriesHandler<'a> {
    election_state: Arc<ElectionState<'a>>,
}

impl AppendEntriesHandler<'_> {
    pub fn init(election_state: Arc<ElectionState>) -> AppendEntriesHandler {
        AppendEntriesHandler { election_state }
    }
}

impl MessageHandler<RaftState> for AppendEntriesHandler<'_> {
    fn make_response_body(&self, message: &JsonValue, curr_state: &RaftState) -> Result<JsonValue, MaelstromError> {
        let body = get_body(message);
        let remote_term = body["term"].as_i32().unwrap();
        self.election_state.maybe_step_down(remote_term);
        let mut response = object!(type: "append_entries_res", term: self.election_state.current_term(), success: false);
        if remote_term < self.election_state.current_term() {
            write_log("Remote term less than current term returning.");
            return Ok(response);
        }
        self.election_state.set_leader(body["leader_id"].as_str().unwrap().into());
        self.election_state.reset_election_time();
        let prev_log_index = body["prev_log_index"].as_i32().unwrap() as usize;
        if prev_log_index <= 0 {
            write_log("Out of bounds on log index");
            let def_error = abort(format!("Out of bounds previous log index: {}", prev_log_index));
            return Err(MaelstromError { in_reply_to: message["id"].as_i32().unwrap(), error: def_error });
        }
        let prev_log_term = body["prev_log_term"].as_i32().unwrap() as usize;
        let entry = curr_state.log_entry(prev_log_index);
        if entry.is_none() || entry.unwrap().term as usize != prev_log_term {
            if entry.is_none() {
                write_log(format!("No entry found at {}", prev_log_index).as_str());
            } else {
                write_log(format!("Entry term of {} did not match prev_log_term of {}", entry.unwrap().term, prev_log_term).as_str());
            }

            return Ok(response);
        }
        curr_state.truncate_log(prev_log_index);
        curr_state.append_log_entries(&mut parse_entries(&body["entries"]));

        let leader_commit = body["leader_commit"].as_i32().unwrap() as usize;
        if self.election_state.commit_index() < leader_commit {
            let new_commit_index = min(leader_commit, curr_state.log_size());
            self.election_state.set_commit_index(new_commit_index);
        }
        response["success"] = JsonValue::from(true);
        return Ok(response);
    }
}