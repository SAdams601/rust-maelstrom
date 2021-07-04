use std::sync::mpsc::SyncSender;
use std::sync::{Mutex, RwLock};
use std::cell::RefCell;
use std::collections::HashMap;
use json::JsonValue;
use crate::message_utils::get_in_response_to;

pub struct NodeState {
    node_id: RwLock<Option<String>>,
    other_ids: RwLock<Vec<String>>,
    msg_id: Mutex<RefCell<i32>>,
    callbacks: RwLock<HashMap<i32, SyncSender<JsonValue>>>,
    response_channel: SyncSender<String>,
}

impl NodeState {

    pub fn init(response_channel: SyncSender<String>) -> NodeState {
        NodeState {
            node_id: RwLock::new(None),
            other_ids: RwLock::new(Vec::new()),
            msg_id: Mutex::new(RefCell::new(0)),
            callbacks: RwLock::new(HashMap::new()),
            response_channel: response_channel
        }
    }

    pub fn get_channel(&self) -> SyncSender<String> {
        self.response_channel.clone()
    }

    pub fn next_msg_id(&self) -> i32 {
        let cell = self.msg_id.lock().unwrap();
        cell.replace_with(|i| *i + 1)
    }

    pub fn node_id(&self) -> String {
        self.node_id.read().unwrap().as_ref().unwrap().clone()
    }

    pub fn is_initialized(&self) -> bool {
        self.node_id.read().unwrap().is_some()
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

    pub fn check_for_callback(&self, message: &JsonValue) -> Option<SyncSender<JsonValue>> {
        let in_response_to = get_in_response_to(message);
        match in_response_to {
            Some(id) => self.callbacks.write().unwrap().remove(&id),
            None => None,
        }
    }

    pub fn add_callback(&self, message_id: i32, channel: SyncSender<JsonValue>) {
        self.callbacks.write().unwrap().insert(message_id, channel);
    }
}
