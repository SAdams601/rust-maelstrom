use shared_lib::message_handler::MessageHandler;
use crate::raft_node_state::RaftState;
use json::{JsonValue, object};
use shared_lib::error::MaelstromError;
use shared_lib::message_utils::get_body;
use shared_lib::stdio::write_log;
use crate::election_state::{RpcCall};
use std::sync::mpsc::{SyncSender, sync_channel};
use crate::election_state::RpcCall::{CurrentTerm, VotedFor, MaybeStepDown, VoteFor};

pub struct RequestVoteHandler {
    election_state_sender: SyncSender<RpcCall>
}

impl RequestVoteHandler {
    pub fn init(election_state_sender: SyncSender<RpcCall>) -> RequestVoteHandler {
        RequestVoteHandler {
            election_state_sender
        }
    }

    fn current_term(&self) -> i32 {
        let (sender, receiver) = sync_channel(1);
        let call = CurrentTerm(sender);
        self.election_state_sender.send(call);
        receiver.recv().unwrap()
    }

    fn voted_for(&self) -> Option<String> {
        let (sender, receiver) = sync_channel(1);
        let call = VotedFor(sender);
        self.election_state_sender.send(call);
        receiver.recv().unwrap()
    }

    fn maybe_step_down(&self, vote_term: i32) -> bool {
        let (sender, receiver) = sync_channel(1);
        let call = MaybeStepDown(vote_term, sender);
        self.election_state_sender.send(call);
        receiver.recv().unwrap()
    }

    fn vote_for(&self, id: String) {
        let call = VoteFor(id);
        self.election_state_sender.send(call);
    }
}

impl MessageHandler<RaftState> for RequestVoteHandler {
    fn make_response_body(&self, message: &JsonValue, curr_state: &RaftState) -> Result<JsonValue, MaelstromError> {
        let mut grant = false;
        let body = get_body(message);

        let vote_term = body["term"].as_i32().unwrap();
        let current_term = self.current_term();

        let voted_for = self.voted_for();

        let vote_log_term = body["last_log_term"].as_i32().unwrap();
        let last_log_term = curr_state.log_last().term;

        let vote_log_size = body["last_log_index"].as_usize().unwrap();
        let log_size = curr_state.log_size();

        self.maybe_step_down(vote_term);
        if vote_term < current_term {
            write_log(format!("Candidate term {} lower than {} not granting vote.", vote_term, current_term).as_str());
        } else if voted_for.is_some() {
            write_log(format!("Have already voted for {} this term", voted_for.unwrap()).as_str());
        } else if vote_log_term < last_log_term {
            write_log(format!("Have log entries for term {} which is newer than remote term {}", last_log_term, vote_log_term).as_str());
        } else if vote_log_term == last_log_term && vote_log_size < log_size {
            write_log(format!("Both logs at term {} but local log is {} and remote is only {}.", last_log_term, log_size, vote_log_size).as_str());
        } else {
            let candidate_id = body["candidate_id"].to_string();
            write_log(format!("Voting for {}", candidate_id).as_str());
            grant = true;
            self.vote_for(candidate_id);
        }
        Ok(object! (type: "request_vote_res", term: current_term, vote_granted: grant))
    }
}