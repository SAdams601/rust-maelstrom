use json::{self, JsonValue};
use lazy_static::lazy_static;
use lin_kv_service::LinKvService;
use message_handlers::{
    add_handler::AddHandler, echo_handler::EchoHandler, init_handler::InitHandler,
    read_handler::ReadHandler, replicate_handler::ReplicateHandler,
    topology_handler::TopologyHandler, txn_handler::TxnHandler,
};

use states::maelstrom_node_state::MaelstromState;
use std::{collections::HashMap, io::prelude::*, sync::mpsc::sync_channel};
use std::{
    io::{self},
    sync::mpsc::Receiver,
};
use std::{
    io::{stderr, Write},
    thread,
};
use shared_lib::{ stdio::while_reply, message_handler::MessageHandler, message_utils::get_message_type};
use shared_lib::read_respond::read_respond_loop;

mod counters;
mod lin_kv_service;
mod message_handlers;
mod states;

lazy_static! {
    static ref MESSAGE_HANDLERS: HashMap<String, Box<dyn MessageHandler<MaelstromState>>> = {
        let mut map: HashMap<String, Box<dyn MessageHandler<MaelstromState>>> = HashMap::new();
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
    static ref NODE_STATE: MaelstromState = {
        let (reply_sender, reply_receiver) = sync_channel(1);
        thread::spawn(|| while_reply(reply_receiver));
        MaelstromState::init(reply_sender)
    };
    static ref LIN_KV_SERVICE: LinKvService = LinKvService::init(&NODE_STATE);
}

fn main() {
    read_respond_loop(&*NODE_STATE, &*MESSAGE_HANDLERS)
}


