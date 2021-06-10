use json::JsonValue;
use std::io;
use std::io::{Write, BufRead};

pub fn read_respond(handle_fn : fn(JsonValue)) {
    for result in io::stdin().lock().lines() {
        match result {
            Ok(line) => {
                io::stderr().write(format!("Received {}\n", line).as_ref());
                let parsed_res = json::parse(&line);
                match parsed_res {
                    Ok(parsed) => {
                        handle_fn(parsed)
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
