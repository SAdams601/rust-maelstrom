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
use std::sync::mpsc::{sync_channel, SyncSender, TryIter, TryRecvError, Receiver};
use shared_lib::stdio::write_log;
use crate::election_state::RpcCall::{MaybeStepDown, VoteGranted};

#[derive(Copy, Clone, PartialEq)]
pub enum State {
    LEADER,
    FOLLOWER,
    CANDIDATE,
}

pub struct ElectionState<'a> {
    next_election: RwLock<Instant>,
    term: RwLock<i32>,
    curr_state: RwLock<State>,
    node_state: &'a RaftState,
    voted_for: RwLock<Option<String>>,
    sender: SyncSender<RpcCall>
}

impl ElectionState<'_> {
    fn init(state: &RaftState, sender: SyncSender<RpcCall>) -> ElectionState {
        ElectionState {
            next_election: RwLock::new(Instant::now()),
            term: RwLock::new(0),
            curr_state: RwLock::new(FOLLOWER),
            node_state: state,
            voted_for: RwLock::new(None),
            sender: sender
        }
    }

    pub fn reset_election_time(&mut self) {
        let mut next_election = *self.next_election.write().unwrap();
        let mut rng = rand::thread_rng();
        let rand: u64 = rng.gen_range(0..10);
        let standard_timeout = Duration::new(2, 0);
        next_election = Instant::now() + (standard_timeout + Duration::from_secs(rand + 1))
    }

    fn advance_term(&self, new_term: i32) -> Result<(), String> {
        let mut curr_term = *self.term.write().unwrap();
        if new_term < curr_term {
            let error_message = format!("Cannot change term from {} to {}\n", curr_term, new_term);
            stderr().write(error_message.as_bytes());
            return Err(error_message);
        }
        curr_term = new_term;
        *self.voted_for.write().unwrap() = None;
        Ok(())
    }

    fn become_candidate(&mut self) {
        let mut curr_state = *self.curr_state.write().unwrap();
        let curr_term = *self.term.read().unwrap();
        curr_state = CANDIDATE;
        self.advance_term(curr_term + 1);
        self.reset_election_time();
        *self.voted_for.write().unwrap() = Some(self.node_state.node_id());
        stderr().write(format!("Becoming candidate at term {}\n", (curr_term.clone() + 1)).as_ref());
        self.request_votes();
    }

    fn become_follower(&self) {
        let mut curr_state = *self.curr_state.write().unwrap();
        let curr_term = self.term.read().unwrap();
        stderr().write(format!("Becoming follower at term {}\n", curr_term).as_ref());
        curr_state = FOLLOWER;
    }

    fn become_leader(&mut self) {
        let mut curr_state = *self.curr_state.write().unwrap();
        let curr_term = self.term.read().unwrap();
        stderr().write(format!("Becoming leader at term {}\n", curr_term).as_ref());
        curr_state = LEADER;
    }

    fn request_votes(&mut self) {
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

    fn maybe_step_down(&self, remote_term: i32) -> bool {
        let term = self.term.write().unwrap();
        if *term < remote_term {
            drop(term);
            self.advance_term(remote_term);
            self.become_follower();
            return false
        }
        false
    }

    fn current_term(&self) -> i32 {
        *self.term.read().unwrap()
    }

    fn voted_for(&self) -> Option<String> {
        match self.voted_for.read().unwrap().as_ref() {
            None => None,
            Some(str) => Some(str.clone())
        }
    }

    fn vote_for(&self, id: String) {
        *self.voted_for.write().unwrap() = Some(id);
    }

    fn vote_granted(&self, body: JsonValue) -> bool {
        let curr_state = *self.curr_state.read().unwrap();
        let curr_term = self.current_term();
        curr_state == CANDIDATE &&
            curr_term == body["term"].as_i32().unwrap() &&
            body["vote_granted"].as_bool().unwrap()
    }

    pub fn handle_rpc_calls(&self, calls: TryIter<RpcCall>) {
        for call in calls {
            match call{
                RpcCall::CurrentTerm(sender) => { sender.send(self.current_term()); },
                RpcCall::VotedFor(sender) => { sender.send(self.voted_for()); },
                RpcCall::MaybeStepDown(remote_term, sender) => { sender.send(self.maybe_step_down(remote_term)); },
                RpcCall::VoteFor(id) => { self.vote_for(id); }
                RpcCall::VoteGranted(body, sender) => { sender.send(self.vote_granted(body)); }
            }
        }
    }
}

pub enum RpcCall {
    CurrentTerm(SyncSender<i32>),
    VotedFor(SyncSender<Option<String>>),
    MaybeStepDown(i32, SyncSender<bool>),
    VoteGranted(JsonValue, SyncSender<bool>),
    VoteFor(String)
}


pub fn election_loop(node_state: &'static RaftState) -> SyncSender<RpcCall> {
    let (sender, receiver) = sync_channel(1);
    let result = sender.clone();
    thread::spawn(move || {
        let mut state = ElectionState::init(node_state, sender.clone());
        loop {
            let messages = receiver.try_iter();
            state.handle_rpc_calls(messages);
            let next_election = *state.next_election.read().unwrap();
            if next_election < Instant::now() {
                let current_state = *state.curr_state.read().unwrap();
                if current_state != LEADER && node_state.is_initialized() {
                    state.become_candidate();
                }
                state.reset_election_time();
            }
            let rand = rand::thread_rng().next_u64() % 20;
            thread::sleep(Duration::from_millis(90 + rand))
        }});
    result
}

fn count_votes(candidate_id: String, state_sender: SyncSender<RpcCall>, receivers: Vec<Receiver<JsonValue>>) {
    thread::spawn(move || {
        let mut votes = HashSet::new();
        votes.insert(candidate_id);
        let mut received = 0;
        let total_votes = receivers.len();
        while received < total_votes {
            for receiver in &receivers {
                let result = receiver.try_recv();
                match result {
                    Ok(msg) => {
                        let body = &msg["body"];
                        let (sender,receiver) = sync_channel(1);
                        state_sender.send(MaybeStepDown(body["term"].as_i32().unwrap(), sender));
                        if receiver.recv().unwrap() {
                            return;
                        }
                        let (sender, receiver) = sync_channel(1);
                        state_sender.send(VoteGranted(body.clone(), sender));
                        if receiver.recv().unwrap() {
                            votes.insert(msg["src"].to_string());
                        }
                    }
                    Err(error) => {
                        match error {
                            TryRecvError::Empty => (),
                            TryRecvError::Disconnected => {
                                write_log("Channel disconnected before response received during broadcast.");
                                received = received + 1;
                            }
                        }
                    }
                }
            }
        }
    });
}
