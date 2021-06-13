mod message_handlers;
mod raft_node_state;

use std::io::{stderr, stdout, BufRead, Write};
use std::sync::mpsc::Receiver;
use std::io;
use shared_lib::read_respond;

fn main() {
    read_respond::read_respond(|jv| {
        io::stderr().write(format!("In loop got {:?}", jv).as_ref());
    });
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
