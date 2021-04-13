use json::{self};
use lazy_static::lazy_static;
use lin_kv_service::LinKvService;
use message_handler::{
    add_handler::AddHandler, echo_handler::EchoHandler, init_handler::InitHandler,
    read_handler::ReadHandler, replicate_handler::ReplicateHandler,
    topology_handler::TopologyHandler, txn_handler::TxnHandler, MessageHandler,
};
use message_utils::get_message_type;

use states::node_state::NodeState;
use std::{collections::HashMap, io::prelude::*, sync::mpsc::sync_channel};
use std::{
    io::{self},
    sync::mpsc::Receiver,
};
use std::{
    io::{stderr, Write},
    thread,
};
mod counters;
mod error;
mod lin_kv_service;
mod message_handler;
mod message_utils;
mod states;

lazy_static! {
    static ref MESSAGE_HANDLERS: HashMap<String, Box<dyn MessageHandler>> = {
        let mut map: HashMap<String, Box<dyn MessageHandler>> = HashMap::new();
        map.insert("init".to_string(), Box::new(InitHandler {}));
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
    static ref NODE_STATE: NodeState = {
        let (reply_sender, reply_receiver) = sync_channel(1);
        thread::spawn(|| while_receive(reply_receiver, write_reply));
        NodeState::init(reply_sender)
    };
    static ref LIN_KV_SERVICE: LinKvService = LinKvService::init(&NODE_STATE);
}

fn main() {
    for result in io::stdin().lock().lines() {
        match result {
            Ok(line) => {
                io::stderr().write(format!("Received {}\n", line).as_ref());
                let parsed_res = json::parse(&line);
                match parsed_res {
                    Ok(parsed) => match NODE_STATE.check_for_callback(&parsed) {
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
                    },
                    Err(err) => {
                        std::process::exit(1);
                    }
                }
            }
            Err(err) => {
                std::process::exit(1);
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
