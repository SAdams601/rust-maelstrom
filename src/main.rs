use std::{borrow::Borrow, cell::RefCell, io::{prelude::*, stderr, stdout}};
use std::io;
use std::io::{Write};
use json::{self, stringify};
use json::JsonValue;

mod node;
use crate::node::{NodeState};


fn write_reply(msg: JsonValue) {
    let mut stdout = io::stdout();
    let str = stringify(msg);
    stderr().write(format!("Sending: {}\n", str).as_bytes());
    stdout.write_all(str.as_bytes());
    stdout.write_all("\n".as_bytes());

    stdout.flush();
}

fn main() {
    let mut is_initialized = false;
    let mut node = NodeState::init("my_id".to_string());    
    for result in io::stdin().lock().lines() {
        match result {
            Ok(line) => {
                io::stderr().write(format!("Received {}\n", line).as_ref());
                let parsed_res = json::parse(&line);                
                match parsed_res {
                    Ok(parsed) => {
                        if !is_initialized {
                            let node_id = parsed["body"]["node_id"].as_str().unwrap();
                            node = NodeState::init(node_id.to_string());
                            is_initialized = true;
                        }                        
                        write_reply(node.respond(parsed));                        
                    }
                    Err(err) => {                        
                        std::process::exit(1);   
                    }
                }                
            },
            Err(err) => {                
                std::process::exit(1);
            }
        }
    }
}
