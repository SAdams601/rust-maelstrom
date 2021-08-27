use shared_lib::message_handler::MessageHandler;
use crate::raft_node_state::RaftState;
use json::{JsonValue, object};
use shared_lib::error::{MaelstromError, DefiniteError, abort};
use crate::election_state::ElectionState;
use std::sync::Arc;
use shared_lib::message_utils::get_body;
use crate::log::{Entry, parse_entries};
use std::cmp::min;

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
        if remote_term >= self.election_state.current_term() {
            self.election_state.reset_election_time();
            let prev_log_index = body["prev_log_index"].as_i32().unwrap();
            if prev_log_index <= 0 {
                let in_reply_to = body["msg_id"].as_i32().unwrap();
                let error = abort(format!("Out of bounds previous log index {}", prev_log_index));
                return Err(MaelstromError { in_reply_to, error })
            }
            let last_entry = curr_state.log_entry(prev_log_index as usize);
            if last_entry.is_some() && last_entry.unwrap().term == body["prev_log_term"].as_i32().unwrap() {
                curr_state.truncate_log(prev_log_index as usize);

                let mut entries: Vec<Entry> = parse_entries(&body["entries"]);
                curr_state.append_log_entries(&mut entries);
                let leader_commit_index = body["leader_commit"].as_i32().unwrap();
                if (self.election_state.commit_index() as i32) < leader_commit_index {
                    self.election_state.set_commit_index(min(curr_state.log_size(), leader_commit_index as usize));
                    response["success"] = JsonValue::from(true);
                }
            }
        }
        Ok(response)
    }
}