use json::JsonValue;
use std::{
    cell::RefCell,
    collections::HashMap,
    sync::{mpsc::SyncSender, Mutex, MutexGuard, RwLock},
};

use crate::{counters::pn_counter::PnCounter, message_utils::get_in_reponse_to};

use super::datomic_state::DatomicState;

pub struct NodeState {
    node_id: RwLock<Option<String>>,
    other_ids: RwLock<Vec<String>>,
    msg_id: Mutex<RefCell<i32>>,
    neighbors: RwLock<Vec<String>>,
    callbacks: RwLock<HashMap<i32, SyncSender<JsonValue>>>,
    counters: RwLock<PnCounter>,
    response_channel: SyncSender<String>,
    datomic_state: Mutex<DatomicState>,
}

impl NodeState {
    pub fn init(response_channel: SyncSender<String>) -> NodeState {
        let ns = NodeState {
            node_id: RwLock::new(None),
            other_ids: RwLock::new(Vec::new()),
            msg_id: Mutex::new(RefCell::new(0)),
            neighbors: RwLock::new(Vec::new()),
            callbacks: RwLock::new(HashMap::new()),
            counters: RwLock::new(PnCounter::init()),
            response_channel: response_channel,
            datomic_state: Mutex::new(DatomicState::init()),
        };
        ns
    }

    pub fn set_node_id(&self, my_id: String) {
        let mut id = self.node_id.write().unwrap();
        id.replace(my_id);
    }

    pub fn set_other_node_ids(&self, other_ids: Vec<&str>) {
        let mut ids = self.other_ids.write().unwrap();
        let my_id: String = self.node_id.read().unwrap().as_ref().unwrap().to_string();
        other_ids.iter().for_each(|id_ref| {
            let id: String = id_ref.to_string();
            if my_id != id {
                ids.push(id);
            }
        });
    }

    pub fn other_nodes(&self) -> Vec<String> {
        self.other_ids.read().unwrap().clone()
    }

    pub fn node_id(&self) -> String {
        self.node_id.read().unwrap().as_ref().unwrap().clone()
    }

    pub fn read_counters(&self) -> i32 {
        self.counters.read().unwrap().read()
    }

    pub fn counters_state(&self) -> JsonValue {
        self.counters.read().unwrap().to_json()
    }

    pub fn next_msg_id(&self) -> i32 {
        let cell = self.msg_id.lock().unwrap();
        cell.replace_with(|i| *i + 1)
    }

    pub fn replace_topology(&self, new_neighbors: Vec<String>) {
        let mut vec = self.neighbors.write().unwrap();
        new_neighbors.iter().for_each(|id| vec.push(id.clone()));
    }

    pub fn get_neighbors(&self) -> Vec<String> {
        self.neighbors
            .read()
            .unwrap()
            .iter()
            .map(|n| n.clone())
            .collect()
    }

    pub fn new_message(&self, message: i32) {
        let mut counters = self.counters.write().unwrap();
        counters.add(self.node_id(), message);
    }

    pub fn merge_messages(&self, received_values: PnCounter) {
        let mut counters = self.counters.write().unwrap();
        counters.merge(received_values);
    }

    pub fn check_for_callback(&self, message: &JsonValue) -> Option<SyncSender<JsonValue>> {
        let in_response_to = get_in_reponse_to(message);
        match in_response_to {
            Some(id) => self.callbacks.write().unwrap().remove(&id),
            None => None,
        }
    }

    pub fn get_channel(&self) -> SyncSender<String> {
        self.response_channel.clone()
    }

    pub fn borrow_datomic(&self) -> MutexGuard<DatomicState> {
        self.datomic_state.lock().unwrap()
    }
}
