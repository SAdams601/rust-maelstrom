use crate::states::{node_state::NodeState, serializable_map::SerializableMap};
use json::{object, stringify, JsonValue};
use std::{
    io::{stderr, Write},
    sync::mpsc::sync_channel,
    time::Duration,
};

pub struct LinKvService {
    state: &'static NodeState,
}

impl LinKvService {
    pub fn init(state: &'static NodeState) -> LinKvService {
        LinKvService { state }
    }

    pub fn read_root(&self) -> SerializableMap {
        let response_body = &self.send_rpc(object! {type: "read", key: "root"})["body"];
        if response_body["type"] == "error" {
            let map = self.init_root();
            return map;
        }
        SerializableMap::from_json(&response_body["value"])
    }

    pub fn cas_root(&self, map: SerializableMap) -> Result<(), String> {
        let response = self.send_rpc(
            object! {type: "cas", key: "root", from: map.original_to_json(), to: map.to_json(), create_if_not_exists: true},
        );
        if response["body"]["type"].to_string() != "cas_ok" {
            stderr().write_all("Cas failed to update root".as_bytes());
            return Err(format!(
                "cas failed with type {}",
                response["body"]["type"].to_string()
            ));
        }
        Ok(())
    }

    fn init_root(&self) -> SerializableMap {
        let map = SerializableMap::init();
        self.send_rpc(object! {type: "write", key: "root", value: map.to_json()});
        map
    }

    fn send_rpc(&self, mut request_body: JsonValue) -> JsonValue {
        let msg_id = self.state.next_msg_id();
        request_body["msg_id"] = JsonValue::from(msg_id);
        let request = object! {dest: "lin-kv", src: self.state.node_id(), body: request_body};
        let (sender, receiver) = sync_channel(1);
        self.state.add_callback(msg_id, sender);
        self.state.get_channel().send(stringify(request));
        receiver.recv_timeout(Duration::from_millis(5000)).unwrap()
    }
}
