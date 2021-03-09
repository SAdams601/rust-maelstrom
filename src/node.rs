use std::{cell::{RefCell}, io::{Write, stderr}};
use json::{JsonValue, object};


    pub struct NodeState {
        pub node_id: String,
        pub msg_id: RefCell<i32>,
    }

    impl NodeState {

        pub fn init(my_id: String) -> NodeState {
            NodeState { node_id: my_id, msg_id: RefCell::new(1) }
        }

        //{"dest":"n1","body":{"type":"init","node_id":"n1","node_ids":["n1"],"msg_id":1},"src":"c0","id":0}

        pub fn respond(&self, message: JsonValue) -> JsonValue {
            let this_msg_id = self.msg_id.take() + 1;
            self.msg_id.replace(this_msg_id);
            let body = &message["body"];
            let msg_type = &body["type"];
            let type_str = msg_type.as_str().unwrap();

            let mut response_body = object! {msg_id : this_msg_id, in_reply_to: body["msg_id"].clone()};
            
            match type_str {
                "init" => {
                    response_body["type"] = JsonValue::from("init_ok");                    
                },
                "echo" => {
                    response_body["type"] = JsonValue::from("echo_ok");
                    response_body["echo"] = body["echo"].clone();
                },
                _ => {
                    let error_str = format!("Unknown message type received <{}>\n", type_str);                    
                    response_body["type"] = JsonValue::from("error");
                    response_body["code"] = JsonValue::from(10);
                    response_body["text"] = JsonValue::from(error_str);
                }
            }
            let from = message["src"].as_str().unwrap();            
            return self.respond_to_with(from, response_body)
        }
        
        
        fn respond_to_with(&self, to: &str, body: JsonValue) -> JsonValue {
            object! { src: self.my_id(), dest: to, body: body}
        }

        fn my_id(&self) -> &str {
            &self.node_id
        }

    }
