use shared_lib::node_state::NodeState;
use std::time::{Duration, Instant};
use std::thread::Thread;
use std::thread;
use std::sync::{RwLock, Arc, LockResult, RwLockReadGuard};
use rand::{Rng, thread_rng};
use crate::election_state::State::{FOLLOWER, LEADER, CANDIDATE};
use std::io::{stderr, Write};
use std::ops::Deref;

#[derive(Copy, Clone, PartialEq)]
pub enum State {
    LEADER,
    FOLLOWER,
    CANDIDATE,
}

pub struct ElectionState {
    next_election: RwLock<Instant>,
    term: RwLock<i32>,
    curr_state: RwLock<State>,
}

impl ElectionState {
    fn init() -> ElectionState {
        ElectionState {
            next_election: RwLock::new(Instant::now()),
            term: RwLock::new(0),
            curr_state: RwLock::new(FOLLOWER),
        }
    }

    fn reset_election_time(&mut self) {
        let mut next_election = *self.next_election.write().unwrap();
        let mut rng = rand::thread_rng();
        let rand: u64 = rng.gen_range(0..10);
        let standard_timeout = Duration::new(2, 0);
        next_election = Instant::now() + (standard_timeout + Duration::from_secs(rand + 1))
    }

    fn advance_term(&mut self, new_term: i32) -> Result<(), String> {
        let mut curr_term = *self.term.write().unwrap();
        if new_term < curr_term {
            let error_message = format!("Cannot change term from {} to {}\n", curr_term, new_term);
            stderr().write(error_message.as_bytes());
            return Err(error_message);
        }
        curr_term = new_term;
        Ok(())
    }

    fn become_candidate(&mut self) {
        let mut curr_state = *self.curr_state.write().unwrap();
        let curr_term = *self.term.read().unwrap();
        curr_state = CANDIDATE;
        self.advance_term(curr_term + 1);
        self.reset_election_time();
        stderr().write(format!("Becoming candidate at term {}\n", (curr_term.clone() + 1)).as_ref());
    }

    fn become_follower(&mut self) {
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
}


pub fn election_loop() {
    thread::spawn( || {
        let mut state = ElectionState::init();
        loop {
            let next_election = *state.next_election.read().unwrap();
            if next_election < Instant::now() {
                let current_state = *state.curr_state.read().unwrap();
                if current_state != LEADER {
                    state.become_candidate();
                }
                state.reset_election_time();
            }
            thread::sleep(Duration::from_secs(1))
        }});
}
