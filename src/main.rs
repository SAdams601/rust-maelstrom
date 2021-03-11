use json::{self};
use std::{io, sync::mpsc::Receiver};
use std::{io::prelude::*, sync::mpsc::channel};
use std::{
    io::{stderr, Write},
    thread,
};

mod node;
use crate::node::NodeState;

fn main() {
    let (log_sender, log_receiver) = channel();
    thread::spawn(|| while_receive(log_receiver, write_log));

    let (reply_sender, reply_receiver) = channel();
    thread::spawn(|| while_receive(reply_receiver, write_reply));

    let mut node = NodeState::init(log_sender, reply_sender);
    for result in io::stdin().lock().lines() {
        match result {
            Ok(line) => {
                io::stderr().write(format!("Received {}\n", line).as_ref());
                let parsed_res = json::parse(&line);
                match parsed_res {
                    Ok(parsed) => {
                        node.respond(parsed);
                    }
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
    stdout.write_all(msg.as_bytes());
    stdout.write_all("\n".as_bytes());
    stdout.flush();
}

fn write_log(msg: String) {
    stderr().write(msg.as_bytes());
    stderr().flush();
}
