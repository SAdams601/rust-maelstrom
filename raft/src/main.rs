mod message_handlers;

use message_handlers::init_handler::InitHandler;

mod raft_node_state;
mod election_state;


use raft_node_state::RaftState;
use lazy_static::lazy_static;
use std::io::{stderr, stdout, BufRead, Write};
use std::sync::mpsc::Receiver;
use std::{io, thread};
use shared_lib::{read_respond, message_handler::MessageHandler};
use std::{collections::HashMap, io::prelude::*, sync::mpsc::sync_channel};
use shared_lib::message_utils::get_message_type;
use json::JsonValue;
use crate::message_handlers::read_handler::ReadHandler;
use crate::message_handlers::cas_handler::CasHandler;
use crate::message_handlers::write_handler::WriteHandler;
use crate::election_state::ElectionState;

lazy_static! {
    static ref MESSAGE_HANDLERS: HashMap<String, Box<dyn MessageHandler<RaftState>>> = {
        let mut map: HashMap<String, Box<dyn MessageHandler<RaftState>>> = HashMap::new();
        map.insert(
            "init".to_string(),
            Box::new(InitHandler::init()),
        );
        map.insert("read".to_string(), Box::new(ReadHandler {}));
        map.insert("cas".to_string(), Box::new(CasHandler {}));
        map.insert("write".to_string(), Box::new(WriteHandler {}));
        map
    };

    static ref NODE_STATE: RaftState = {
        let (reply_sender, reply_receiver) = sync_channel(1);
        thread::spawn(|| while_receive(reply_receiver, write_reply));
        RaftState::init(reply_sender)
    };
}

fn main() {
    election_state::election_loop();
    read_respond::read_respond( message_handler)
}

fn message_handler(parsed: JsonValue) {
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
