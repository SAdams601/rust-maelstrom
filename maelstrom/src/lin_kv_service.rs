use std::{
    collections::HashMap,
    io::{stderr, Write},
    sync::{mpsc::sync_channel, Mutex},
    thread,
    time::Duration,
};

use json::{JsonValue, object, stringify};
use shared_lib::error::DefiniteError;
use shared_lib::node_state::NodeState;
use shared_lib::rpc::{retry_rpc, send_rpc};
use crate::states::{kv_thunk::KVValue, maelstrom_node_state::MaelstromState, serializable_map::SerializableMap, thunk::Thunk};
use std::borrow::{Borrow, BorrowMut};

pub struct LinKvService {
    state: &'static MaelstromState,
    cache: Mutex<HashMap<String, JsonValue>>,
    root: Mutex<Thunk<SerializableMap>>,
}

impl LinKvService {
    pub fn init(state: &'static MaelstromState) -> LinKvService {
        LinKvService {
            state,
            cache: Mutex::new(HashMap::new()),
            root: Mutex::new(Thunk::init(
                "init_root".to_string(),
                Some(SerializableMap::from_json(&JsonValue::new_object())),
                false,
            )),
        }
    }

    pub fn read_root(&self) -> Thunk<SerializableMap> {
        self.root.lock().unwrap().clone()
    }

    pub fn update_root(&self) {
        let mut root = self.root.lock().unwrap();
        let jv = retry_rpc(self.state, object! {type: "read", key: "root"}.borrow_mut());
        *root = Thunk::init(jv["value"].to_string(), None, true);
    }

    pub fn init_root(&self) -> Thunk<SerializableMap> {
        let map = SerializableMap::init();
        let thunk = Thunk::init(self.state.next_thunk_id(), Some(map), false);
        thunk.save(self);
        send_rpc(self.state, &mut object! {type: "write", key: "root", value: thunk.id.clone()});
        let mut root = self.root.lock().unwrap();
        *root = thunk.clone();
        thunk
    }

    pub fn cas_root(&self, original_id: String, new_id: String) -> Result<(), DefiniteError> {
        let response = send_rpc(self.state,
            &mut object! {type: "cas", key: "root", from: original_id.clone(), to: new_id, create_if_not_exists: true},
        );
        if response["body"]["type"].to_string() != "cas_ok" {
            stderr()
                .write_all(format!("Cas failed to update root at {}\n", original_id).as_bytes());
            return Err(shared_lib::error::txn_conflict("cas root failed".to_string()));
        }
        Ok(())
    }

    pub fn read_thunk_json<T: KVValue>(&self, thunk: &Thunk<T>) -> JsonValue {
        let mut cache = self.cache.lock().unwrap();
        if cache.contains_key(&thunk.id) {
            let value = cache.get(&thunk.id).unwrap();
            stderr().write_all(
                format!(
                    "Reading {}, from cache {:?}\n",
                    thunk.id.clone(),
                    value.clone()
                )
                    .as_bytes(),
            );
            return value.clone();
        }
        let json = retry_rpc(self.state, object! {type: "read", key: thunk.id.clone()}.borrow_mut())["value"].clone();
        cache.insert(thunk.id.clone(), json.clone());
        json
    }

    pub fn save_thunk<T: KVValue>(&self, thunk: &Thunk<T>) -> JsonValue {
        let thunk_json = thunk.value(self).to_json();
        let mut cache = self.cache.lock().unwrap();
        cache.insert(thunk.id.clone(), thunk_json.clone());
        let mut request = object! {type: "write", key: thunk.id.clone(), value: thunk_json};
        send_rpc(self.state, &mut request)
    }

    pub fn new_id(&self) -> String {
        self.state.next_thunk_id()
    }
}
