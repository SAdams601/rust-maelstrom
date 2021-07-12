use std::time::{Duration, Instant};
use std::thread::Thread;
use std::thread;
use std::sync::{RwLock, Arc, LockResult, RwLockReadGuard, PoisonError};
use rand::{Rng, thread_rng, random, RngCore};
use std::io::{stderr, Write};
use std::ops::{Deref};
use std::collections::HashSet;
use json::{object, JsonValue};
use shared_lib::{node_state::NodeState, rpc::broadcast_rpc};
use crate::election_state::State::{FOLLOWER, LEADER, CANDIDATE};
use crate::raft_node_state::RaftState;
use std::borrow::Borrow;
use lazy_static::lazy_static;
use std::sync::mpsc::{sync_channel, SyncSender, TryIter, TryRecvError, Receiver, channel};
use shared_lib::stdio::write_log;
use crate::election_state::RpcCall::{MaybeStepDown, VoteGranted, NextElection, ResetElectionTime, ValidateElection, ResetStepDownTime, StepDownDeadline};

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum State {
    LEADER,
    FOLLOWER,
    CANDIDATE,
}

pub struct ElectionState<'a> {
    next_election: RwLock<Instant>,
    step_down: RwLock<Instant>,
    term: RwLock<i32>,
    curr_state: RwLock<State>,
    node_state: &'a RaftState,
    voted_for: RwLock<Option<String>>,
    sender: SyncSender<RpcCall>,
}

impl ElectionState<'_> {
    fn init(state: &RaftState, sender: SyncSender<RpcCall>) -> ElectionState {
        ElectionState {
            next_election: RwLock::new(Instant::now()),
            step_down: RwLock::new(Instant::now()),
            term: RwLock::new(0),
            curr_state: RwLock::new(FOLLOWER),
            node_state: state,
            voted_for: RwLock::new(None),
            sender,
        }
    }

    fn reset_election_time(&self) {
        let mut next_election = self.next_election.write().unwrap();
        let mut rng = rand::thread_rng();
        let rand: u64 = rng.gen_range(0..10);
        let standard_timeout = Duration::new(2, 0);
        *next_election = Instant::now() + (standard_timeout + Duration::from_secs(rand + 1))
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

    fn become_candidate(&self) {
        let mut curr_state = self.curr_state.write().unwrap();
        let curr_term = *self.term.read().unwrap();
        *curr_state = CANDIDATE;
        self.advance_term(curr_term + 1);
        self.reset_election_time();
        self.reset_step_down_time();
        *self.voted_for.write().unwrap() = Some(self.node_state.node_id());
        write_log(format!("Becoming candidate at term {}\n", (curr_term.clone() + 1)).as_ref());
        self.request_votes();
    }

    fn become_follower(&self) {
        let mut curr_state = self.curr_state.write().unwrap();
        let curr_term = self.term.read().unwrap();
        write_log(format!("Becoming follower at term {}\n", curr_term).as_ref());
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
        write_log(format!("Becoming leader at term {}\n", curr_term).as_ref());
        *curr_state = LEADER;
    }

    fn request_votes(&self) {
        let candidate_id = self.node_state.node_id();
        let term = self.term.read().unwrap().clone();
        let mut request = object! {type: "request_vote",
                                             term: term,
                                             candidate_id: candidate_id.clone(),
                                             last_log_index: self.node_state.log_size(),
                                             last_log_term: self.node_state.log_last().term
                                            };
        let receivers = broadcast_rpc(self.node_state, &mut request);
        count_votes(candidate_id.clone(), self.sender.clone(), receivers);
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

    pub(crate) fn current_term(&self) -> i32 {
        *self.term.read().unwrap()
    }

    fn next_election_time(&self) -> Instant {
        *self.next_election.read().unwrap()
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

    fn vote_granted(&self, body: JsonValue) -> bool {
        let curr_state = *self.curr_state.read().unwrap();
        let curr_term = self.current_term();
        let result = curr_state == CANDIDATE &&
            curr_term == body["term"].as_i32().unwrap() &&
            body["vote_granted"].as_bool().unwrap();
        write_log(format!("Checking vote, state: {:?} term: {} body_term: {}, granted: {}, result: {}", curr_state, curr_term, body["term"], body["vote_granted"], result).as_str());
        result
    }

    fn current_state(&self) -> State {
        (*self.curr_state.read().unwrap()).clone()
    }

    fn validate_election(&self, votes: HashSet<String>) {
        let majority = self.node_state.majority();
        if majority <= votes.len() as i32 {
            self.become_leader();
        }
    }

    fn step_down_time(&self) -> Instant {
        *self.step_down.read().unwrap()
    }

    pub fn handle_rpc_calls(&self, call: RpcCall) {
        match call {
            RpcCall::CurrentTerm(sender) => { sender.send(self.current_term()); }
            RpcCall::VotedFor(sender) => { sender.send(self.voted_for()); }
            RpcCall::MaybeStepDown(remote_term, sender) => { sender.send(self.maybe_step_down(remote_term)); }
            RpcCall::VoteFor(id) => { self.vote_for(id); }
            RpcCall::VoteGranted(body, sender) => { sender.send(self.vote_granted(body)); }
            RpcCall::ResetElectionTime => { self.reset_election_time(); }
            RpcCall::ResetStepDownTime => { self.reset_step_down_time(); }
            RpcCall::NextElection(sender) => { sender.send(self.next_election_time()); }
            RpcCall::ValidateElection(votes) => { self.validate_election(votes) }
            RpcCall::StepDownDeadline(sender) => { sender.send(self.step_down_time()); }
        }
    }
}

pub enum RpcCall {
    CurrentTerm(SyncSender<i32>),
    ResetElectionTime,
    ResetStepDownTime,
    StepDownDeadline(SyncSender<Instant>),
    NextElection(SyncSender<Instant>),
    VotedFor(SyncSender<Option<String>>),
    MaybeStepDown(i32, SyncSender<bool>),
    VoteGranted(JsonValue, SyncSender<bool>),
    VoteFor(String),
    ValidateElection(HashSet<String>),
}

fn rpc_loop(node_state: &'static RaftState, sender: SyncSender<RpcCall>, receiver: Receiver<RpcCall>) {
    thread::spawn(move || {
        let state = ElectionState::init(node_state, sender);
        loop {
            let call = receiver.recv().unwrap();
            state.handle_rpc_calls(call);
        }
    });
}

pub fn start(node_state: &'static RaftState) -> Arc<ElectionState> {
    let (sender, receiver) = sync_channel(50);
    let thread_copy = sender.clone();
    let state = ElectionState::init(node_state, thread_copy);
    let state_arc = Arc::new(state);
    let result = Arc::clone(&state_arc);
    thread::spawn(move || {
        start_elections(node_state, &state_arc);
        step_down_loop(&state_arc);
        loop {
            let call = receiver.recv().unwrap();
            state_arc.handle_rpc_calls(call);
        }
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

//do not start until node state is initialized!
pub fn election_loop(state_arc: &Arc<ElectionState<'static>>) {
    let election_state = Arc::clone(state_arc);
    loop {
        let rand = rand::thread_rng().next_u64() % 100;
        thread::sleep(Duration::from_millis(50 + rand));
        let next_election = election_state.next_election_time();
        let current_state = election_state.current_state();
        if next_election < Instant::now() && current_state != LEADER {
            election_state.become_candidate();
        }
        election_state.reset_election_time();
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

fn count_votes(candidate_id: String, state_sender: SyncSender<RpcCall>, receivers: Vec<Receiver<JsonValue>>) {
    thread::spawn(move || {
        state_sender.send(ResetStepDownTime);
        let mut votes = HashSet::new();
        let mut have_voted: Vec<usize> = Vec::new();
        votes.insert(candidate_id);
        let mut received = 0;
        let total_votes = receivers.len();
        while received < total_votes {
            for (i, receiver) in receivers.iter().enumerate() {
                if !have_voted.contains(&i) {
                    let result = receiver.try_recv();
                    match result {
                        Ok(msg) => {
                            let body = &msg["body"];
                            let (sender, receiver) = sync_channel(1);
                            state_sender.send(MaybeStepDown(body["term"].as_i32().unwrap(), sender));
                            if receiver.recv().unwrap() {
                                return;
                            }
                            let (sender, receiver) = sync_channel(1);
                            state_sender.send(VoteGranted(body.clone(), sender));
                            if receiver.recv().unwrap() {
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
        }
        write_log(format!("Have votes: {:?}", votes).as_str());
        state_sender.send(ValidateElection(votes));
    });
}
