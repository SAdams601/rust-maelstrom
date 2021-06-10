use json::{self, JsonValue};
use lazy_static::lazy_static;
use lin_kv_service::LinKvService;
use message_handlers::{
    add_handler::AddHandler, echo_handler::EchoHandler, init_handler::InitHandler,
    read_handler::ReadHandler, replicate_handler::ReplicateHandler,
    topology_handler::TopologyHandler, txn_handler::TxnHandler,
};
use message_utils::get_message_type;

use states::maelstrom_node_state::MaelstromNodeState;
use std::{collections::HashMap, io::prelude::*, sync::mpsc::sync_channel};
use std::{
    io::{self},
    sync::mpsc::Receiver,
};
use std::{
    io::{stderr, Write},
    thread,
};
use shared_lib::read_respond::read_respond;
use shared_lib::message_handler::MessageHandler;
mod counters;
mod lin_kv_service;
mod message_handlers;
mod message_utils;
mod states;

lazy_static! {
    static ref MESSAGE_HANDLERS: HashMap<String, Box<dyn MessageHandler<State = MaelstromNodeState>>> = {
        let mut map: HashMap<String, Box<dyn MessageHandler<State = MaelstromNodeState>>> = HashMap::new();
        map.insert(
            "init".to_string(),
            Box::new(InitHandler::init(&LIN_KV_SERVICE)),
        );
        map.insert("echo".to_string(), Box::new(EchoHandler {}));
        map.insert("read".to_string(), Box::new(ReadHandler {}));
        map.insert("topology".to_string(), Box::new(TopologyHandler {}));
        map.insert("add".to_string(), Box::new(AddHandler {}));
        map.insert("replicate".to_string(), Box::new(ReplicateHandler {}));
        map.insert(
            "txn".to_string(),
            Box::new(TxnHandler::init(&LIN_KV_SERVICE)),
        );
        map
    };
    static ref NODE_STATE: MaelstromNodeState = {
        let (reply_sender, reply_receiver) = sync_channel(1);
        thread::spawn(|| while_receive(reply_receiver, write_reply));
        MaelstromNodeState::init(reply_sender)
    };
    static ref LIN_KV_SERVICE: LinKvService = LinKvService::init(&NODE_STATE);
}

fn main() {
    read_respond(message_handler);
}

fn message_handler(parsed: JsonValue) {
    match NODE_STATE.check_for_callback(&parsed) {
        Some(sender) => {
            sender.send(parsed);
        }
        None => {
            let message_type: String = get_message_type(&parsed);
            if MESSAGE_HANDLERS.contains_key(&message_type) {
                let handler = MESSAGE_HANDLERS.get(&message_type).unwrap();
                thread::spawn(move || handler.handle_message(&parsed, &NODE_STATE));
            } else {
                write_log(&format!(
                    "Did not find handler for message: {:?}",
                    parsed
                ));
            }
        }
    }
}

fn while_receive<F: Fn(String)>(receiver: Receiver<String>, f: F) {
    while let Ok(text) = receiver.recv() {
        f(text);
    }
}

fn write_reply(msg: String) {
    let mut stdout = io::stdout();
    if !msg.contains("\"dest\":\"lin-kv\"") {
        write_log(&format!("Replying: {}", msg));
    }
    stdout.write_all(msg.as_bytes());
    stdout.write_all("\n".as_bytes());
    stdout.flush();
}

fn write_log(msg: &str) {
    stderr().write(msg.as_bytes());
    stderr().write(b"\n");
    stderr().flush();
}
