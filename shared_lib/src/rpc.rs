use crate::node_state::NodeState;
use crate::stdio::write_log;
use json::{JsonValue, stringify, object};
use std::sync::mpsc::{sync_channel, TryRecvError, Receiver, RecvTimeoutError};
use std::time::Duration;
use std::thread;
use std::borrow::Borrow;
use std::thread::JoinHandle;

pub fn send_rpc(state: &NodeState, request_body: &mut JsonValue, to: &str) -> Option<JsonValue> {
    let msg_id = state.next_msg_id();
    request_body["msg_id"] = JsonValue::from(msg_id);
    let request =
        object! {dest: to, src: state.node_id(), body: request_body.clone()};
    let (sender, receiver) = sync_channel(1);
    state.add_callback(msg_id, sender);
    state.get_channel().send(stringify(request));
    let response = receiver.recv_timeout(Duration::from_millis(5000));
    match response {
        Ok(jv) => Some(jv),
        Err(err) => {
            write_log("RPC timout error.");
            None
        }
    }
}

pub fn retry_rpc(state: &NodeState, request_body: &mut JsonValue) -> JsonValue {
    while let rpc_response = send_rpc(state, request_body, "lin-kv").unwrap() {
        if rpc_response["body"]["type"] != "error" {
            return rpc_response["body"].clone();
        }
        thread::sleep(Duration::from_millis(10));
    }
    JsonValue::from("Should not be here!")
}

pub fn broadcast_rpc(state: &NodeState, request_body: &mut JsonValue) -> Vec<Receiver<JsonValue>> {
    let other_nodes = state.other_nodes();
    let mut receivers = Vec::new();
    for node in other_nodes {
        let msg_id = state.next_msg_id();
        request_body["msg_id"] = JsonValue::from(msg_id);
        let request =
            object! {dest: node, src: state.node_id(), body: request_body.clone()};
        let (sender, receiver) = sync_channel(1);
        state.add_callback(msg_id, sender);
        state.get_channel().send(stringify(request));
        receivers.push(receiver);
    }
    receivers
}