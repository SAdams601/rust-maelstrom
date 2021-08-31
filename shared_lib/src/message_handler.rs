use json::{JsonValue, stringify, object};
use crate::{error::MaelstromError, node_state::NodeState};
use std::ops::Deref;

pub trait MessageHandler<T>: Sync
    where T: Deref<Target=NodeState> {
    fn make_response_body(
        &self,
        message: &JsonValue,
        curr_state: &T,
    ) -> Result<JsonValue, MaelstromError>;

    fn handle_message(&self, message: &JsonValue, curr_state: &T) {
        let response = self.get_response_body(message, curr_state);
        if response.is_err() {
            let error = response.expect_err("");
            let error_body = construct_error_body(&error);
            let response = wrap_response_body(
                error_body,
                JsonValue::from(error.in_reply_to),
                id_from(&message).clone(),
                curr_state.next_msg_id(),
                curr_state.node_id(),
            );
            curr_state.get_channel().send(stringify(response));
            return;
        }
        let maybe_response_body = response.expect("");
        if maybe_response_body.is_some() {
            let response_body = maybe_response_body.unwrap();
            let response = wrap_response_body(
                response_body,
                message["body"]["msg_id"].clone(),
                id_from(&message).clone(),
                curr_state.next_msg_id(),
                curr_state.node_id(),
            );
            curr_state.get_channel().send(stringify(response));
        }
    }

    fn get_response_body(
        &self,
        message: &JsonValue,
        curr_state: &T,
    ) -> Result<Option<JsonValue>, MaelstromError> {
        self.make_response_body(message, curr_state)
            .map(|body| Some(body))
    }
}

pub fn construct_error_body(error: &MaelstromError) -> JsonValue {
    object! {type: "error", in_reply_to: error.in_reply_to, code: error.error.code, text: error.error.text.clone()}
}

fn wrap_response_body(
    mut response_body: JsonValue,
    msg_replying_to: JsonValue,
    dest: JsonValue,
    msg_id: i32,
    node_id: String,
) -> JsonValue {
    response_body["in_reply_to"] = msg_replying_to;
    response_body["msg_id"] = JsonValue::from(msg_id);
    object!(dest: dest, src: JsonValue::from(node_id), body: response_body)
}

fn id_from(message: &JsonValue) -> &JsonValue {
    &message["src"]
}
