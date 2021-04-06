use json::{object, JsonValue};

use crate::node::NodeState;

use super::MessageHandler;
pub struct TopologyHandler {}

impl MessageHandler for TopologyHandler {
    fn make_response_body(&self, message: &JsonValue, curr_state: &NodeState) -> JsonValue {
        let mut neighbors: Vec<String> = Vec::new();
        message["body"]["topology"][curr_state.node_id()]
            .members()
            .for_each(|jv| neighbors.push(jv.to_string()));
        curr_state.replace_topology(neighbors);
        object! (type: "topology_ok")
    }
}