use crate::node_state::NodeState;
use crate::stdio::write_log;
use json::{JsonValue, stringify, object};
use std::sync::mpsc::{sync_channel, TryRecvError};
use std::time::Duration;
use std::thread;
use std::borrow::Borrow;

pub fn send_rpc(state: &NodeState, request_body: &mut JsonValue) -> JsonValue {
    let msg_id = state.next_msg_id();
    request_body["msg_id"] = JsonValue::from(msg_id);
    let request =
        object! {dest: "lin-kv", src: state.node_id(), body: request_body.clone()};
    let (sender, receiver) = sync_channel(1);
    state.add_callback(msg_id, sender);
    state.get_channel().send(stringify(request));
    receiver.recv_timeout(Duration::from_millis(5000)).unwrap()
}

pub fn retry_rpc(state: &NodeState, request_body: &mut JsonValue) -> JsonValue {
    while let rpc_response = send_rpc(state, request_body) {
        if rpc_response["body"]["type"] != "error" {
            return rpc_response["body"].clone();
        }
        thread::sleep(Duration::from_millis(10));
    }
    JsonValue::from("Should not be here!")
}

pub fn broadcast_rpc(state: &NodeState, request_body: &mut JsonValue) -> Vec<JsonValue> {
    let other_nodes = state.other_nodes();
    let mut receivers = Vec::new();
    let mut responses = Vec::new();
    for node in other_nodes {
        let msg_id = state.next_msg_id();
        request_body["msg_id"] = JsonValue::from(msg_id);
        let request =
            object! {dest: "lin-kv", src: state.node_id(), body: request_body.clone()};
        let (sender, receiver) = sync_channel(1);
        state.add_callback(msg_id, sender);
        state.get_channel().send(stringify(request));
        receivers.push(receiver);
    }
    let mut received = 0;
    while received < receivers.len() {
        for receiver in &receivers {
            let result = receiver.try_recv();
            match result {
                Ok(msg) => {
                    responses.push(msg);
                    received = received + 1;
                }
                Err(error) => {
                    match error {
                        TryRecvError::Empty => (),
                        TryRecvError::Disconnected => {
                            write_log("Channel disconnected before response received during broadcast.");
                            received = received + 1;
                        }
                    }
                }
            }
        }
    }
    responses
}