use std::collections::HashSet;

use json::JsonValue;

use crate::{
    counters::{g_counter::GCounter, pn_counter::PnCounter},
    node::NodeState,
};

use super::MessageHandler;

pub struct ReplicateHandler {}

impl MessageHandler for ReplicateHandler {
    fn make_response_body(&self, message: &JsonValue, curr_state: &NodeState) -> json::JsonValue {
        unimplemented!("Replicate Handler uses get_response_body")
    }

    fn get_response_body(&self, message: &JsonValue, curr_state: &NodeState) -> Option<JsonValue> {
        let counters = PnCounter::from_json(&message["body"]["value"]);
        curr_state.merge_messages(counters);
        None
    }
}
