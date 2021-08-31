use json::JsonValue;
use std::{io, thread};
use std::io::{Write, BufRead, stderr};
use crate::node_state::NodeState;
use std::collections::HashMap;
use crate::message_handler::MessageHandler;
use std::ops::Deref;
use crate::message_utils::get_message_type;
use crate::stdio::write_log;

pub fn read_respond_loop<T>(state: &'static T, handlers : &'static HashMap<String, Box<dyn MessageHandler<T>>>)
    where T: Deref<Target = NodeState> + Sync
{
    for result in io::stdin().lock().lines() {
        match result {
            Ok(line) => {
                io::stderr().write(format!("Received {}\n", line).as_ref());
                let parsed_res = json::parse(&line);
                match parsed_res {
                    Ok(parsed) => {
                        match state.check_for_callback(&parsed) {
                            Some(sender) => {
                                sender.send(parsed);
                            }
                            None => {
                                let message_type: String = get_message_type(&parsed);
                                if handlers.contains_key(&message_type) {
                                    let handler = handlers.get(&message_type).unwrap();
                                    thread::spawn(move || handler.handle_message(&parsed, state));
                                } else {
                                    write_log(&format!(
                                        "Did not find handler for message: {}",
                                        parsed
                                    ));
                                }
                            }
                        }
                    },
                    Err(_err) => {
                        std::process::exit(1);
                    }
                }
            }
            Err(_err) => {
                std::process::exit(1);
            }
        }
    }
}

