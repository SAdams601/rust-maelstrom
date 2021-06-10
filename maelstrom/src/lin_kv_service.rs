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
use crate::states::{kv_thunk::KVValue, maelstrom_node_state::MaelstromNodeState, serializable_map::SerializableMap, thunk::Thunk};

pub struct LinKvService {
    state: &'static MaelstromNodeState,
    cache: Mutex<HashMap<String, JsonValue>>,
    root: Mutex<Thunk<SerializableMap>>,
}

impl LinKvService {
    pub fn init(state: &'static MaelstromNodeState) -> LinKvService {
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
        let jv = &self.retry_rpc(object! {type: "read", key: "root"})["value"];
        *root = Thunk::init(jv.to_string(), None, true);
    }

    pub fn init_root(&self) -> Thunk<SerializableMap> {
        let map = SerializableMap::init();
        let thunk = Thunk::init(self.state.next_thunk_id(), Some(map), false);
        thunk.save(self);
        self.send_rpc(&mut object! {type: "write", key: "root", value: thunk.id.clone()});
        let mut root = self.root.lock().unwrap();
        *root = thunk.clone();
        thunk
    }

    pub fn cas_root(&self, original_id: String, new_id: String) -> Result<(), DefiniteError> {
        let response = self.send_rpc(
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
        let json = self.retry_rpc(object! {type: "read", key: thunk.id.clone()})["value"].clone();
        cache.insert(thunk.id.clone(), json.clone());
        json
    }

    pub fn save_thunk<T: KVValue>(&self, thunk: &Thunk<T>) -> JsonValue {
        let thunk_json = thunk.value(self).to_json();
        let mut cache = self.cache.lock().unwrap();
        cache.insert(thunk.id.clone(), thunk_json.clone());
        let mut request = object! {type: "write", key: thunk.id.clone(), value: thunk_json};
        self.send_rpc(&mut request)
    }

    pub fn new_id(&self) -> String {
        self.state.next_thunk_id()
    }

    fn retry_rpc(&self, mut request_body: JsonValue) -> JsonValue {
        while let rpc_response = self.send_rpc(&mut request_body) {
            if rpc_response["body"]["type"] != "error" {
                return rpc_response["body"].clone();
            }
            thread::sleep(Duration::from_millis(10));
        }
        JsonValue::from("Should not be here!")
    }

    fn send_rpc(&self, request_body: &mut JsonValue) -> JsonValue {
        let msg_id = self.state.next_msg_id();
        request_body["msg_id"] = JsonValue::from(msg_id);
        let request =
            object! {dest: "lin-kv", src: self.state.node_id(), body: request_body.clone()};
        let (sender, receiver) = sync_channel(1);
        self.state.add_callback(msg_id, sender);
        self.state.get_channel().send(stringify(request));
        receiver.recv_timeout(Duration::from_millis(5000)).unwrap()
    }
}
