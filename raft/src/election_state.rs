use std::time::{Duration, Instant};
use std::thread::Thread;
use std::thread;
use std::sync::{RwLock, Arc, LockResult, RwLockReadGuard, PoisonError};
use rand::{Rng, thread_rng, random, RngCore};
use std::io::{stderr, Write};
use std::ops::{Deref};
use std::collections::{HashSet, HashMap};
use json::{object, JsonValue};
use shared_lib::{node_state::NodeState, rpc::{broadcast_rpc, send_rpc}};
use crate::election_state::State::{FOLLOWER, LEADER, CANDIDATE};
use crate::raft_node_state::RaftState;
use std::borrow::Borrow;
use lazy_static::lazy_static;
use std::sync::mpsc::{sync_channel, SyncSender, TryIter, TryRecvError, Receiver, channel};
use shared_lib::stdio::write_log;
use shared_lib::message_utils::get_body;
use std::cmp::max;
use crate::log::Op;
use crate::message_handlers::cas_handler::check_cas_result;
use crate::message_handlers::read_handler::check_read_result;
use shared_lib::error::MaelstromError;
use shared_lib::message_handler::construct_error_body;
use shared_lib::rpc::send_ff;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum State {
    LEADER,
    FOLLOWER,
    CANDIDATE,
}

pub struct ElectionState<'a> {
    next_election: RwLock<Instant>,
    step_down: RwLock<Instant>,
    replication_time: RwLock<Instant>,
    term: RwLock<i32>,
    curr_state: RwLock<State>,
    node_state:  &'a RaftState,
    voted_for: RwLock<Option<String>>,
    leader: RwLock<Option<String>>,
    commit_index: RwLock<usize>,
    last_applied: RwLock<usize>,
    next_index: RwLock<HashMap<String, usize>>,
    match_index: RwLock<HashMap<String, usize>>
}

impl ElectionState<'_> {
    fn init(state: &RaftState) -> ElectionState {
        ElectionState {
            next_election: RwLock::new(Instant::now()),
            step_down: RwLock::new(Instant::now()),
            replication_time: RwLock::new(Instant::now()),
            term: RwLock::new(0),
            curr_state: RwLock::new(FOLLOWER),
            node_state: state,
            voted_for: RwLock::new(None),
            leader: RwLock::new(None),
            commit_index: RwLock::new(0),
            last_applied: RwLock::new(1),
            next_index: RwLock::new(HashMap::new()),
            match_index: RwLock::new(HashMap::new())
        }
    }

    pub fn reset_election_time(&self) {
        let mut next_election = self.next_election.write().unwrap();
        let mut rng = rand::thread_rng();
        let rand: u64 = rng.gen_range(0..10);
        let standard_timeout = Duration::new(2, 0);
        *next_election = Instant::now() + (standard_timeout + Duration::from_secs(rand + 1))
    }

    pub fn commit_index(&self) -> usize {
        *self.commit_index.read().unwrap()
    }

    pub fn set_commit_index(&self, i: usize) {
        let mut index = self.commit_index.write().unwrap();
        *index = i;
    }

    fn reset_step_down_time(&self) {
        let mut step_down_time = self.step_down.write().unwrap();
        let mut rng = rand::thread_rng();
        let rand: u64 = rng.gen_range(0..10);
        let standard_timeout = Duration::new(2, 0);
        *step_down_time = Instant::now() + (standard_timeout + Duration::from_secs(rand + 1))
    }

    fn advance_term(&self, new_term: i32) -> Result<(), String> {
        let mut curr_term = self.term.write().unwrap();
        if new_term < *curr_term {
            let error_message = format!("Cannot change term from {} to {}\n", curr_term, new_term);
            stderr().write(error_message.as_bytes());
            return Err(error_message);
        }
        *curr_term = new_term;
        let mut voted_for = self.voted_for.write().unwrap();
        *voted_for = None;
        Ok(())
    }

    fn become_candidate(&self) -> Vec<Receiver<JsonValue>> {
        let mut curr_state = self.curr_state.write().unwrap();
        let curr_term = *self.term.read().unwrap();
        *curr_state = CANDIDATE;
        self.advance_term(curr_term + 1);
        self.reset_election_time();
        self.reset_step_down_time();
        *self.voted_for.write().unwrap() = Some(self.node_state.node_id());
        *self.leader.write().unwrap() = None;
        write_log(format!("Becoming candidate at term {}\n", (curr_term.clone() + 1)).as_ref());
        self.request_votes()
    }

    fn become_follower(&self) {
        let mut curr_state = self.curr_state.write().unwrap();
        let curr_term = self.term.read().unwrap();
        write_log(format!("Becoming follower at term {}\n", curr_term).as_ref());
        self.reset_election_time();
        *self.voted_for.write().unwrap() = None;
        self.clear_indices();
        *self.leader.write().unwrap() = None;
        *curr_state = FOLLOWER;
    }

    fn become_leader(&self) {
        let mut curr_state = self.curr_state.write().unwrap();
        let curr_term = self.term.read().unwrap();
        if CANDIDATE != *curr_state {
            write_log(format!("Tried to become leader while state was {:?} at term {}", curr_state, curr_term).as_str());
            return;
        }
        self.reset_step_down_time();
        let mut next_idx = self.next_index.write().unwrap();
        let mut match_idx = self.match_index.write().unwrap();
        let current_log_size = self.node_state.log_size();
        *self.leader.write().unwrap() = None;
        for other_node in self.node_state.other_nodes() {
            next_idx.insert(other_node.clone(), current_log_size + 1);
            match_idx.insert(other_node.clone(), 0);
        }
        write_log(format!("Becoming leader at term {}\n", curr_term).as_ref());
        *curr_state = LEADER;
    }

    fn request_votes(&self) -> Vec<Receiver<JsonValue>> {
        let candidate_id = self.node_state.node_id();
        let term = self.term.read().unwrap().clone();
        let mut request = object! {type: "request_vote",
                                             term: term,
                                             candidate_id: candidate_id.clone(),
                                             last_log_index: self.node_state.log_size(),
                                             last_log_term: self.node_state.log_last().term
                                            };
        broadcast_rpc(self.node_state, &mut request)
    }

    pub(crate) fn maybe_step_down(&self, remote_term: i32) -> bool {
        let term = self.term.write().unwrap();
        write_log(format!("Might Step down term: {}, remote term {}", term, remote_term).as_str());
        if *term < remote_term {
            drop(term);
            self.advance_term(remote_term);
            self.become_follower();
            write_log("Stepping down");
            return true;
        }
        false
    }

    pub fn set_leader(&self, leader_id: String) {
        self.leader.write().unwrap().insert(leader_id);
    }

    pub fn get_leader(&self) -> Option<String> {
        self.leader.read().unwrap().clone()
    }

    pub(crate) fn current_term(&self) -> i32 {
        *self.term.read().unwrap()
    }

    fn next_election_time(&self) -> Instant {
        *self.next_election.read().unwrap()
    }

    fn clear_indices(&self) {
        let mut match_idx = self.match_index.write().unwrap();
        match_idx.clear();
        let mut next_idx = self.next_index.write().unwrap();
        next_idx.clear();
    }

    pub(crate) fn voted_for(&self) -> Option<String> {
        match self.voted_for.read().unwrap().as_ref() {
            None => None,
            Some(str) => Some(str.clone())
        }
    }

    pub(crate) fn vote_for(&self, id: String) {
        *self.voted_for.write().unwrap() = Some(id);
    }

    pub fn next_index_of_node(&self, node_id: &str) -> usize {
        *self.next_index.read().unwrap().get(node_id).unwrap_or(&0)
    }

    pub fn match_index_of_node(&self, node_id: &str) -> usize {
        *self.match_index.read().unwrap().get(node_id).unwrap_or(&0)
    }

    pub fn set_node_next_index(&self, node_id: &str, i: usize) {
        let mut map = self.next_index.write().unwrap();
        map.insert(node_id.to_string(), i);
    }

    pub fn set_node_match_index(&self, node_id: &str, i: usize) {
        let mut map = self.match_index.write().unwrap();
        map.insert(node_id.to_string(), i);
    }

    fn vote_granted(&self, body: JsonValue) -> bool {
        let curr_state = *self.curr_state.read().unwrap();
        let curr_term = self.current_term();
        let result = curr_state == CANDIDATE &&
            curr_term == body["term"].as_i32().unwrap() &&
            body["vote_granted"].as_bool().unwrap();
        write_log(format!("Checking vote, state: {:?} term: {} body_term: {}, granted: {}, result: {}", curr_state, curr_term, body["term"], body["vote_granted"], result).as_str());
        result
    }

    pub fn advance_state_machine(&self) {
        let last_applied = *self.last_applied.read().unwrap();
        let commit_index = *self.commit_index.read().unwrap();
        for index in (last_applied+1).. (last_applied + commit_index) {
            let entry = self.node_state.log_entry(index);
            if entry.is_none() {
                continue;
            }
            let m_op = entry.unwrap().op;
            if m_op.is_none() {
                continue;
            }
            let op = m_op.unwrap();
            write_log(format!("Applying op: {:?}", op).as_str());
            let (to, response) = match op {
                Op::CAS { key, from,to, requester , msg_id} => {
                    let cas_result = self.node_state.cas_value(key, from, to);
                    (requester, check_cas_result(cas_result, msg_id))
                },
                Op::Read { key, requester , msg_id} => {
                    let read_result = self.node_state.read_value(key);
                    (requester, check_read_result(read_result, msg_id))
                },
                Op::Write { key, value,requester, msg_id } => {
                    self.node_state.write_value(key, value);
                    (requester, Ok(object!(type: "write_ok", in_reply_to: msg_id)))
                }
            };
            if self.current_state() == LEADER {
                match response {
                    Ok(mut jv) => {
                        write_log(format!("Sending RPC from state machine: {}", jv).as_str());
                        send_ff(self.node_state, &mut jv, &to);
                    }
                    Err(err) => {
                        let mut error_json = construct_error_body(&err);
                        send_ff(self.node_state, &mut error_json, &to);
                    }
                }
            }
        }
        drop(last_applied);
        *self.last_applied.write().unwrap() = commit_index;
    }

    pub(crate) fn current_state(&self) -> State {
        (*self.curr_state.read().unwrap()).clone()
    }

    fn validate_election(&self, votes: &HashSet<String>) -> bool {
        write_log(format!("Validating election with votes {:?}", votes).as_str());
        let majority = self.node_state.majority();
        if majority <= votes.len() as i32 {
            self.become_leader();
            return true;
        }
        return false;
    }

    fn median_commit_index(&self) -> usize {
        let current_match_indices = self.match_index.read().unwrap();
        let mut values: Vec<&usize> = current_match_indices.values().collect();
        values.sort();
        let majority = self.node_state.majority() as usize;
        *values[(values.len() - majority)]
    }

    fn advance_commit_index(&self) {
        if self.current_state() == LEADER {
            let n = self.median_commit_index();
            write_log(format!("Median commit index is: {}", n).as_str());
            if self.commit_index() < n && self.node_state.log_entry(n).unwrap().term == self.current_term() {
                self.set_commit_index(n);
            }
        }
        self.advance_state_machine();
    }

    fn step_down_time(&self) -> Instant {
        *self.step_down.read().unwrap()
    }

    pub fn candidate_id(&self) -> String {
        self.node_state.node_id()
    }
}


pub fn start(node_state: &'static RaftState) -> Arc<ElectionState<'static>> {
    let state = ElectionState::init(node_state);
    let state_arc = Arc::new(state);
    let result = Arc::clone(&state_arc);
    thread::spawn(move || {
        start_elections(node_state, &state_arc);
        step_down_loop(&state_arc);
        replicate_log(state_arc);
    });
    result
}

fn start_elections(node_state: &'static RaftState, state_arc: &Arc<ElectionState<'static>>) {
    let arc = Arc::clone(state_arc);
    thread::spawn(move || {
        loop {
            if node_state.is_initialized() {
                election_loop(&arc);
            }
            thread::sleep(Duration::from_millis(10));
        }
    });
}

pub fn election_loop(state_arc: &Arc<ElectionState<'static>>) {
    let election_state = Arc::clone(state_arc);
    loop {
        let rand = rand::thread_rng().next_u64() % 100;
        thread::sleep(Duration::from_millis(50 + rand));
        let next_election = election_state.next_election_time();
        let current_state = election_state.current_state();
        if next_election < Instant::now() && current_state != LEADER {
            let vote_channels = election_state.become_candidate();
            count_votes(Arc::clone(state_arc), vote_channels);
        }
    }
}

pub fn step_down_loop(state_arc: &Arc<ElectionState<'static>>) {
    let election_state = Arc::clone(state_arc);
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(100));
            let current_state = election_state.current_state();
            let step_down_deadline = election_state.step_down_time();
            if current_state == LEADER && step_down_deadline < Instant::now() {
                election_state.become_follower();
            }
        }
    });
}

fn count_votes(election_state: Arc<ElectionState<'static>>, receivers: Vec<Receiver<JsonValue>>) {
    thread::spawn(move || {
        election_state.reset_step_down_time();
        let mut votes = HashSet::new();
        let mut have_voted: Vec<usize> = Vec::new();
        votes.insert(election_state.candidate_id());
        let mut received = 0;
        let total_votes = receivers.len();
        while received < total_votes {
            for (i, receiver) in receivers.iter().enumerate() {
                if !have_voted.contains(&i) {
                    let result = receiver.try_recv();
                    match result {
                        Ok(msg) => {
                            let body = &msg["body"];
                            let stepping_down = election_state.maybe_step_down(body["term"].as_i32().unwrap());
                            if stepping_down {
                                return;
                            }
                            let vote_granted = election_state.vote_granted(body.clone());
                            if vote_granted {
                                votes.insert(msg["src"].to_string());
                                have_voted.push(i);
                                received = received + 1;
                            }
                        }
                        Err(error) => {
                            match error {
                                TryRecvError::Empty => (),
                                TryRecvError::Disconnected => {
                                    write_log("Channel disconnected before response received during election.");
                                    have_voted.push(i);
                                    received = received + 1;
                                }
                            }
                        }
                    }
                }
            }
            if votes.len() >= election_state.node_state.majority() as usize {
                break;
            }
        }
        write_log(format!("Have votes: {:?}", votes).as_str());
        election_state.validate_election(&votes);
    });
}

fn replicate_log(election_state: Arc<ElectionState<'static>>) {
    thread::spawn(move || {
        let mut last_replication = Instant::now();
        let min_replication_interval = Duration::from_millis(50);
        let heartbeat_interval = Duration::from_secs(1);
        loop {
            let mut replicated = false;
            if election_state.current_state() == LEADER {
                let time_since_replication = Instant::now() - last_replication;

                if time_since_replication > min_replication_interval {
                    for other_node in election_state.node_state.other_nodes() {
                        let next_index = election_state.next_index_of_node(&other_node);
                        let entries = election_state.node_state.log_from_index(next_index);
                        let entries_len = entries.len();
                        if entries_len > 0 || heartbeat_interval < time_since_replication {
                            write_log(format!("Replicating {} to {}", next_index, other_node).as_str());
                            replicated = true;
                            let commit_index = election_state.commit_index();

                            let mut message = object! {
                                type: "append_entries",
                                term: election_state.current_term(),
                                leader_id: election_state.node_state.node_id(),
                                prev_log_index: next_index - 1,
                                prev_log_term: election_state.node_state.log_entry(next_index - 1).map_or_else(|| 1, |entry| entry.term),
                                entries: entries,
                                leader_commit: commit_index
                            };
                            let new_arc = election_state.clone();
                            thread::spawn(move || {
                                let thread_state = new_arc;
                                let response = send_rpc(thread_state.node_state, &mut message, &other_node);
                                if response.is_none() {
                                    write_log("Append log failed with a RPC timeout");
                                    return;
                                }
                                let response_value = response.unwrap();
                                let response_body = get_body(&response_value);
                                if response_body["term"].as_i32().is_none() {
                                    write_log(format!("Append entries failed with message: {}", response_body["text"].as_str().unwrap()).as_str());
                                    return;
                                }
                                let response_term = response_body["term"].as_i32().unwrap();
                                thread_state.maybe_step_down(response_term);
                                if thread_state.current_state() == LEADER && response_term == thread_state.current_term() {
                                    thread_state.reset_step_down_time();
                                    if response_body["success"].as_bool().unwrap() {
                                        let new_next_index = max(next_index + entries_len, thread_state.next_index_of_node(&other_node));
                                        thread_state.set_node_next_index(&other_node, new_next_index);
                                        let new_match_index = max(next_index + entries_len - 1, thread_state.match_index_of_node(&other_node));
                                        thread_state.set_node_match_index(&other_node, new_match_index);
                                        write_log(format!("Node: {} next index: {} match index: {}", other_node, new_next_index, new_match_index).as_str());
                                        thread_state.advance_commit_index();
                                    } else {
                                        thread_state.set_node_next_index(&other_node, thread_state.next_index_of_node(&other_node) - 1);
                                    }
                                }
                            });
                        }
                    }
                }
            }
            if replicated {
                last_replication = Instant::now();
            }
            thread::sleep(min_replication_interval);
        }
    });
}
