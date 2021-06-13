use json::{JsonValue, stringify, object};
use crate::{error::MaelstromError, node_state::NodeState};
use std::ops::Deref;

pub trait MessageHandler<T>: Sync
    where T: Deref<Target = NodeState> {

    fn make_response_body(
        &self,
        message: &JsonValue,
        curr_state: &T,
    ) -> Result<JsonValue, MaelstromError>;

    fn handle_message(&self, message: &JsonValue, curr_state: &T) {
        let response = self.get_response_body(message, curr_state);
        if response.is_err() {
            let error = response.expect_err("");
            let error_body = object! {type: "error", in_reply_to: error.in_reply_to, code: error.error.code, text: error.error.text};
            let response = self.wrap_response_body(
                error_body,
                curr_state,
                JsonValue::from(error.in_reply_to),
                self.id_from(&message).clone(),
            );
            curr_state.get_channel().send(stringify(response));
            return;
        }
        let maybe_response_body = response.expect("");
        if maybe_response_body.is_some() {
            let response_body = maybe_response_body.unwrap();
            let response = self.wrap_response_body(
                response_body,
                curr_state,
                message["body"]["msg_id"].clone(),
                self.id_from(&message).clone(),
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

    fn id_from<'a>(&self, message: &'a JsonValue) -> &'a JsonValue {
        &message["src"]
    }

    fn wrap_response_body(
        &self,
        mut response_body: JsonValue,
        state: &T,
        msg_replying_to: JsonValue,
        dest: JsonValue,
    ) -> JsonValue {
        response_body["in_reply_to"] = msg_replying_to;
        response_body["msg_id"] = JsonValue::from(state.next_msg_id());
        object!(dest: dest, src: JsonValue::from(state.node_id()), body: response_body)
    }
}
