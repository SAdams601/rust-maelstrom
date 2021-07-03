use super::id_gen::IdGenerator;
use crate::counters::pn_counter::PnCounter;
use json::JsonValue;
use std::{
    cell::RefCell,
    collections::HashMap,
    sync::{mpsc::SyncSender, Mutex, RwLock},
};
use shared_lib::{node_state::NodeState};
use std::sync::RwLockWriteGuard;
use std::ops::Deref;

pub struct MaelstromState {
    node_state : NodeState,
    neighbors: RwLock<Vec<String>>,

    counters: RwLock<PnCounter>,
    id_gen: RwLock<Option<IdGenerator>>,
}

impl MaelstromState {
    pub fn init(response_channel: SyncSender<String>) -> MaelstromState {
        MaelstromState {
            node_state: NodeState::init(response_channel),
            neighbors: RwLock::new(Vec::new()),
            counters: RwLock::new(PnCounter::init()),
            id_gen: RwLock::new(None),
        }
    }

    pub fn next_thunk_id(&self) -> String {
        let gen = self.id_gen.read().unwrap();
        if gen.is_none() {
            panic!(format!(
                "Tried to get id generator but it has not been inialized. Id is: {}",
                self.node_id()
            ));
        }
        gen.as_ref().unwrap().get_next_id()
    }

    pub fn read_counters(&self) -> i32 {
        self.counters.read().unwrap().read()
    }

    pub fn counters_state(&self) -> JsonValue {
        self.counters.read().unwrap().to_json()
    }

    pub fn replace_topology(&self, new_neighbors: Vec<String>) {
        let mut vec = self.neighbors.write().unwrap();
        new_neighbors.iter().for_each(|id| vec.push(id.clone()));
    }

    pub fn new_message(&self, message: i32) {
        let mut counters = self.counters.write().unwrap();
        counters.add(self.node_id(), message);
    }

    pub fn merge_messages(&self, received_values: PnCounter) {
        let mut counters = self.counters.write().unwrap();
        counters.merge(received_values);
    }
}

impl Deref for MaelstromState {
    type Target = NodeState;

    fn deref(&self) -> &Self::Target {
        &self.node_state
    }
}
