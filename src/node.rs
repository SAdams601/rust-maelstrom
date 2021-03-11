use json::{object, stringify, Array, JsonValue};
use std::{
    borrow::Borrow,
    cell::RefCell,
    collections::HashSet,
    sync::{mpsc::Sender, Mutex},
};

pub struct NodeState {
    node_id: RefCell<String>,
    msg_id: RefCell<i32>,
    other_nodes: Vec<String>,
    neighbors: Vec<String>,
    messages: Mutex<HashSet<i32>>,
    logging_channel: Sender<String>,
    reply_channel: Sender<String>,
}

impl NodeState {
    pub fn init(logging_channel: Sender<String>, reply_channel: Sender<String>) -> NodeState {
        NodeState {
            node_id: RefCell::new("".to_string()),
            msg_id: RefCell::new(1),
            other_nodes: Vec::new(),
            neighbors: Vec::new(),
            messages: Mutex::new(HashSet::new()),
            logging_channel: logging_channel,
            reply_channel: reply_channel,
        }
    }

    //{"dest":"n1","body":{"type":"init","node_id":"n1","node_ids":["n1"],"msg_id":1},"src":"c0","id":0}

    pub fn respond(&mut self, message: JsonValue) {
        let body = &message["body"];
        let msg_type = &body["type"];
        let type_str = msg_type.as_str().unwrap();

        let mut handler = self.get_message_handler(type_str);

        handler(message);
    }

    fn get_message_handler(&mut self, msg_type: &str) -> Box<dyn FnMut(JsonValue) + '_> {
        match msg_type {
            "init" => Box::new(move |jv: JsonValue| {
                let body = jv["body"].borrow();
                self.node_id
                    .replace(body["node_id"].as_str().unwrap().to_string());
                body["node_ids"].members().for_each(|member| {
                    self.other_nodes.push(member.as_str().unwrap().to_string());
                });
                self.reply(jv, object! {type: "init_ok"});
            }),
            "topology" => Box::new(move |jv| {
                jv["body"]["topology"][self.my_id()]
                    .members()
                    .for_each(|neighbor| {
                        self.neighbors.push(neighbor.to_string());
                    });
                self.logging_channel
                    .send(format!("My neighbors are: <{:?}>\n", self.neighbors));
                self.reply(jv, object! {type: "topology_ok"});
            }),
            "read" => Box::new(move |jv: JsonValue| {
                let mtx = self.messages.lock();
                let arr: Vec<JsonValue> = match mtx {
                    Ok(guard) => guard.iter().map(|s| JsonValue::from(s.clone())).collect(),
                    Err(poison_set) => {
                        self.logging_channel
                            .send(format!("Mutex has been poisoned, error: {:?}", poison_set));
                        poison_set
                            .into_inner()
                            .iter()
                            .map(|s| JsonValue::from(s.clone()))
                            .collect()
                    }
                };
                let response = object! {type: "read_ok", messages: arr };
                self.reply(jv, response);
            }),
            "broadcast" => Box::new(move |jv: JsonValue| {
                let message = jv["body"]["message"].as_i32().unwrap();
                if !self.message_seen(message) {
                    let mtx = self.messages.lock();
                    match mtx {
                        Ok(mut guard) => {
                            let set: &mut HashSet<i32> = &mut guard;
                            set.insert(message);
                        }
                        Err(poisoned) => {
                            self.logging_channel
                                .send(format!("Mutex has been poisoned, error: {:?}", poisoned));
                            let set: &mut HashSet<i32> = &mut poisoned.into_inner();
                            set.insert(message);
                        }
                    }
                    self.neighbors.iter().for_each(|node| {
                        self.send(
                            node.clone(),
                            object!(message: message.clone(), type: "broadcast"),
                        )
                    });
                }
                if !jv["body"]["msg_id"].is_null() {
                    self.reply(jv, object!(type: "broadcast_ok"));
                }
            }),
            "echo" => Box::new(move |jv: JsonValue| {
                let echo = jv["body"]["echo"].as_str().unwrap().to_string();
                self.reply(jv, object! {type: "echo_ok", echo: echo});
            }),
            _ => Box::new(move |jv: JsonValue| {
                self.logging_channel.send(format!(
                    "Received message with unknown type message: \n<{}>\n",
                    jv
                ));
            }),
        }
    }

    fn message_seen(&self, message: i32) -> bool {
        self.messages.lock().unwrap().contains(&message)
    }

    fn get_message_id(&self) -> i32 {
        let this_msg_id = self.msg_id.take() + 1;
        self.msg_id.replace(this_msg_id);
        return this_msg_id;
    }

    fn my_id(&self) -> String {
        let id = self.node_id.borrow();
        return id.to_string();
    }

    fn send(&self, neighbor: String, body: JsonValue) {
        let response = object! {
            src: self.my_id(),
            dest: JsonValue::from(neighbor),
            body: body
        };
        self.reply_channel.send(stringify(response));
    }

    fn reply(&self, req_message: JsonValue, mut response_body: JsonValue) {
        let message_id = self.get_message_id();
        let replying_to = req_message["body"]["msg_id"].clone();
        response_body["in_reply_to"] = replying_to;
        response_body["msg_id"] = JsonValue::from(message_id);
        let response = object! {
            dest: req_message["src"].as_str().unwrap(),
            src: self.my_id(),
            body: response_body
        };
        let string = stringify(response);
        self.logging_channel
            .send(format!("Replying with {}\n", string));
        self.reply_channel.send(string);
    }
}
