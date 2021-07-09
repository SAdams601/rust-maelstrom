mod message_handlers;
mod raft_node_state;
mod election_state;
mod log;

use raft_node_state::RaftState;
use message_handlers::init_handler::InitHandler;
use lazy_static::lazy_static;
use std::io::{stderr, stdout, BufRead, Write};
use std::sync::mpsc::SyncSender;
use std::{io, thread};
use shared_lib::{read_respond::read_respond_loop, message_handler::MessageHandler, stdio::while_reply};
use std::{collections::HashMap, io::prelude::*, sync::mpsc::sync_channel};
use shared_lib::message_utils::get_message_type;
use json::JsonValue;
use crate::message_handlers::read_handler::ReadHandler;
use crate::message_handlers::cas_handler::CasHandler;
use crate::message_handlers::write_handler::WriteHandler;
use crate::message_handlers::request_vote_handler::RequestVoteHandler;
use crate::election_state::{ElectionState, RpcCall};

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
        map.insert("request_vote".to_string(), Box::new(RequestVoteHandler::init(ELECTION_STATE_SENDER.clone())));
        map
    };

    static ref NODE_STATE: RaftState = {
        let (reply_sender, reply_receiver) = sync_channel(1);
        thread::spawn(|| while_reply(reply_receiver));
        RaftState::init(reply_sender)
    };

    static ref ELECTION_STATE_SENDER: SyncSender<RpcCall> = {election_state::election_loop(&*NODE_STATE)};
}

fn main() {
    read_respond_loop(&*NODE_STATE, &*MESSAGE_HANDLERS);
}
