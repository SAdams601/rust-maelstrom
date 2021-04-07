pub mod add_handler;
pub mod echo_handler;
pub mod init_handler;
pub mod read_handler;
pub mod replicate_handler;
pub mod topology_handler;

use json::{object, stringify, JsonValue};

use crate::node::NodeState;

pub trait MessageHandler: Sync {
    fn make_response_body(&self, message: &JsonValue, curr_state: &NodeState) -> JsonValue;

    fn handle_message(&self, message: &JsonValue, curr_state: &NodeState) {
        let maybe_response_body = self.get_response_body(message, curr_state);
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

    fn get_response_body(&self, message: &JsonValue, curr_state: &NodeState) -> Option<JsonValue> {
        Some(self.make_response_body(message, curr_state))
    }

    fn id_from<'a>(&self, message: &'a JsonValue) -> &'a JsonValue {
        &message["src"]
    }

    fn wrap_response_body(
        &self,
        mut response_body: JsonValue,
        state: &NodeState,
        msg_replying_to: JsonValue,
        dest: JsonValue,
    ) -> JsonValue {
        response_body["in_reply_to"] = msg_replying_to;
        response_body["msg_id"] = JsonValue::from(state.next_msg_id());
        object!(dest: dest, src: JsonValue::from(state.node_id()), body: response_body)
    }
}
