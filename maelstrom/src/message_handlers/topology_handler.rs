use json::{object, JsonValue};
use shared_lib::{error::MaelstromError, message_handler::MessageHandler, node_state::NodeState};
use crate::{states::maelstrom_node_state::MaelstromState};

pub struct TopologyHandler {}

impl MessageHandler<MaelstromState> for TopologyHandler {

    fn make_response_body(
        &self,
        message: &JsonValue,
        curr_state: &MaelstromState,
    ) -> Result<JsonValue, MaelstromError> {
        let mut neighbors: Vec<String> = Vec::new();
        message["body"]["topology"][curr_state.node_id()]
            .members()
            .for_each(|jv| neighbors.push(jv.to_string()));
        curr_state.replace_topology(neighbors);
        Ok(object! (type: "topology_ok"))
    }
}
