use std::{
    sync::mpsc::{sync_channel, Receiver, SyncSender},
    thread,
    time::Duration,
};

use json::{object, stringify, JsonValue};

use super::MessageHandler;
use crate::{message_utils::get_body, node::NodeState};

pub struct BroadcastHandler {}

impl MessageHandler for BroadcastHandler {
    fn get_response_body(&self, message: &JsonValue, curr_state: &NodeState) -> Option<JsonValue> {
        let body = get_body(message);
        let from = message["src"].to_string();
        let message_value = body["message"].as_i32().unwrap();
        if !curr_state.message_seen(message_value) {
            curr_state.new_message(message_value);
            broadcast_message(from, message_value, curr_state);
            return Some(object! (type: "broadcast_ok"));
        }
        return None;
    }

    fn make_response_body(
        &self,
        _message: &json::JsonValue,
        _curr_state: &NodeState,
    ) -> json::JsonValue {
        unimplemented!("Response for broadcast is handled by get_response_body")
    }
}

fn broadcast_message(message_from: String, message_value: i32, curr_state: &NodeState) {
    let neighbors = curr_state.get_neighbors();
    let common_body = object! (type: "broadcast", message: message_value);
    for neighbor in neighbors {
        if neighbor != message_from {
            let mut body = common_body.clone();
            let next_msg_id = curr_state.next_msg_id();
            body["msg_id"] = JsonValue::from(next_msg_id);
            let message = object! (src: curr_state.node_id(), body: body, dest: neighbor);
            let (sender, reciever) = sync_channel(1);
            let broadcast_channel = curr_state.get_channel().clone();
            thread::spawn(move || handle_ack(message, reciever, broadcast_channel));
            curr_state.add_callback(next_msg_id, sender);
        }
    }
}

fn handle_ack(message: JsonValue, reciever: Receiver<JsonValue>, sender: SyncSender<String>) {
    loop {
        match reciever.try_recv() {
            Ok(_ack) => return,
            Err(_) => {
                sender.send(stringify(message.clone()));
                thread::sleep(Duration::from_millis(1000));
            }
        }
    }
}
