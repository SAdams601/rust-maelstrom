use json::JsonValue;

pub fn get_message_type(message: &JsonValue) -> String {
    get_body(message)["type"].as_str().unwrap().to_string()
}

pub fn get_body(message: &JsonValue) -> &JsonValue {
    &message["body"]
}

pub fn get_in_response_to(message: &JsonValue) -> Option<i32> {
    message["body"]["in_reply_to"].as_i32()
}
