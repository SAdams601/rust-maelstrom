use std::io::{stderr, Write, stdout};
use std::sync::mpsc::Receiver;

pub fn while_reply(receiver: Receiver<String>) {
    while_receive(receiver, write_reply);
}

fn while_receive<F: Fn(String)>(receiver: Receiver<String>, f: F) {
    while let Ok(text) = receiver.recv() {
        f(text);
    }
}

fn write_reply(msg: String) {
    let mut stdout = stdout();
    if !msg.contains("\"dest\":\"lin-kv\"") {
        write_log(&format!("Replying: {}", msg));
    }
    stdout.write_all(msg.as_bytes());
    stdout.write_all("\n".as_bytes());
    stdout.flush();
}

pub fn write_log(msg: &str) {
    stderr().write(msg.as_bytes());
    stderr().write(b"\n");
    stderr().flush();
}