use json::JsonValue;
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    sync::{mpsc::SyncSender, Mutex, RwLock},
};

use crate::message_utils::get_in_reponse_to;

pub struct NodeState {
    node_id: RwLock<Option<String>>,
    msg_id: Mutex<RefCell<i32>>,
    neighbors: RwLock<Vec<String>>,
    messages: RwLock<HashSet<i32>>,
    callbacks: RwLock<HashMap<i32, SyncSender<JsonValue>>>,
    response_channel: SyncSender<String>,
}

impl NodeState {
    pub fn init(response_channel: SyncSender<String>) -> NodeState {
        let ns = NodeState {
            node_id: RwLock::new(None),
            msg_id: Mutex::new(RefCell::new(0)),
            neighbors: RwLock::new(Vec::new()),
            messages: RwLock::new(HashSet::new()),
            callbacks: RwLock::new(HashMap::new()),
            response_channel: response_channel,
        };
        ns
    }

    pub fn set_node_id(&self, my_id: String) {
        let mut id = self.node_id.write().unwrap();
        id.replace(my_id);
    }

    pub fn node_id(&self) -> String {
        self.node_id.read().unwrap().as_ref().unwrap().clone()
    }

    pub fn read_messages(&self) -> Vec<i32> {
        self.messages
            .read()
            .unwrap()
            .iter()
            .map(|i_ref| *i_ref)
            .collect()
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

    pub fn message_seen(&self, message: i32) -> bool {
        self.messages.read().unwrap().contains(&message)
    }

    pub fn new_message(&self, message: i32) {
        let mut messages = self.messages.write().unwrap();
        messages.insert(message);
    }

    pub fn check_for_callback(&self, message: &JsonValue) -> Option<SyncSender<JsonValue>> {
        let in_response_to = get_in_reponse_to(message);
        match in_response_to {
            Some(id) => self.callbacks.write().unwrap().remove(&id),
            None => None,
        }
    }

    pub fn add_callback(&self, message_id: i32, channel: SyncSender<JsonValue>) {
        self.callbacks.write().unwrap().insert(message_id, channel);
    }

    pub fn get_channel(&self) -> SyncSender<String> {
        self.response_channel.clone()
    }
}
