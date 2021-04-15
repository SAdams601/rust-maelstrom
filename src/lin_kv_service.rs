use crate::{
    error::DefiniteError,
    states::{
        kv_thunk::KVValue, node_state::NodeState, serializable_map::SerializableMap, thunk::Thunk,
    },
};
use json::{object, stringify, JsonValue};
use std::{
    io::{stderr, Write},
    sync::mpsc::sync_channel,
    thread,
    time::Duration,
};

pub struct LinKvService {
    state: &'static NodeState,
}

impl LinKvService {
    pub fn init(state: &'static NodeState) -> LinKvService {
        LinKvService { state }
    }

    pub fn read_root(&self) -> Thunk<SerializableMap> {
        let response_body = &self.retry_rpc(object! {type: "read", key: "root"});
        Thunk::init(response_body["value"].to_string(), None, true)
    }

    pub fn init_root(&self) -> Thunk<SerializableMap> {
        let map = SerializableMap::init();
        let thunk = Thunk::init(self.state.next_thunk_id(), Some(map), false);
        thunk.save(self);
        self.send_rpc(&mut object! {type: "write", key: "root", value: thunk.id.clone()});
        thunk
    }

    pub fn cas_root(&self, original_id: String, new_id: String) -> Result<(), DefiniteError> {
        let response = self.send_rpc(
            &mut object! {type: "cas", key: "root", from: original_id, to: new_id, create_if_not_exists: true},
        );
        if response["body"]["type"].to_string() != "cas_ok" {
            stderr().write_all("Cas failed to update root".as_bytes());
            return Err(crate::error::txn_conflict(format!(
                "cas failed with type {}",
                response["body"]["type"].to_string()
            )));
        }
        Ok(())
    }

    pub fn read_thunk_json<T: KVValue>(&self, thunk: &Thunk<T>) -> JsonValue {
        self.retry_rpc(object! {type: "read", key: thunk.id.clone()})["value"].clone()
    }

    pub fn save_thunk<T: KVValue>(&self, thunk: &Thunk<T>) -> JsonValue {
        let mut request =
            object! {type: "write", key: thunk.id.clone(), value: thunk.value(self).to_json()};
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
            object! {dest: "lww-kv", src: self.state.node_id(), body: request_body.clone()};
        let (sender, receiver) = sync_channel(1);
        self.state.add_callback(msg_id, sender);
        self.state.get_channel().send(stringify(request));
        receiver.recv_timeout(Duration::from_millis(5000)).unwrap()
    }
}
