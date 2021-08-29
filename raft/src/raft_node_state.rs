use shared_lib::node_state::NodeState;
use std::sync::mpsc::SyncSender;
use std::sync::{Mutex, RwLock, RwLockReadGuard};
use std::ops::Deref;
use std::collections::HashMap;
use shared_lib::error::{key_does_not_exist, DefiniteError, precondition_failed};
use std::io::{stderr, Write};
use crate::election_state::ElectionState;
use crate::log::{Log, Entry, Op};
use std::thread;
use shared_lib::stdio::write_log;

pub struct RaftState {
    node_state: NodeState,
    values: Mutex<HashMap<i32, i32>>,
    log: RwLock<Option<Log>>,
}

impl RaftState {
    pub fn init(response_channel: SyncSender<String>) -> RaftState {
        RaftState {
            node_state: NodeState::init(response_channel),            values: Mutex::new(HashMap::new()),
            log: RwLock::new(None),
        }
    }

    pub fn read_value(&self, key: i32) -> Result<i32, DefiniteError> {
        let map = self.values.lock().unwrap();
        let m_value = map.get(&key);
        match m_value {
            None => Err(key_does_not_exist(format!("No key found at {}", key))),
            Some(value) => Ok(*value)
        }
    }

    pub fn write_value(&self, key: i32, value: i32) {
        let mut map = self.values.lock().unwrap();
        map.insert(key, value);
    }

    pub fn cas_value(&self, key: i32, from: i32, to: i32) -> Result<(), DefiniteError> {
        self.read_value(key).and_then(|original_value| -> Result<(), DefiniteError> {
            if original_value != from {
                return Err(precondition_failed(format!("Expected {} but value was actually {} at key {}", from, original_value, key)));
            }
            let mut map = self.values.lock().unwrap();
            map.insert(key, to);
            return Ok(());
        })
    }

    pub fn init_log(&self, node_id: String) {
        let mut log = self.log.write().unwrap();
        log.replace(Log::init(node_id));
    }

    pub fn log_size(&self) -> usize {
        match self.log.read().unwrap().as_ref() {
            None => 0,
            Some(log) => log.size()
        }
    }

    pub fn append_single_entry(&self, op: Op, term: i32) {
        let entry = Entry {op: Some(op), term };
        let mut log_loc = self.log.write().unwrap();
        log_loc.as_mut().map(|mut log| {
            let mut vec = vec![entry];
            log.append(&mut vec);
        });
    }

    pub fn append_log_entries(&self, entries: &mut Vec<Entry>) {
        self.log.write().unwrap().as_mut().map(|mut log| log.append(entries));

    }

    pub fn truncate_log(&self, len: usize) {
        self.log.write().unwrap().as_mut().map(|mut log| log.truncate(len));
    }

    pub fn log_entry(&self, i: usize) -> Option<Entry> {
        self.log.read().unwrap().as_ref().and_then(|log| {
          Some(log.get(i))
        })
    }

    pub fn majority(&self) -> i32 {
        (self.other_nodes().len() as i32 / 2) + 1
    }

    pub fn log_last(&self) -> Entry {
        self.log.read().unwrap().as_ref().unwrap().last()
    }

    pub fn log_from_index(&self, i: usize) -> Vec<Entry> { self.log.read().unwrap().as_ref().and_then(|log| {
        write_log(format!("log_from_index - count: {} - log: {:?}", i, log).as_str());
        Some(log.upto_index(i))
    }).unwrap_or(vec![]) }
}

impl Deref for RaftState {
    type Target = NodeState;

    fn deref(&self) -> &Self::Target {
        &self.node_state
    }
}
