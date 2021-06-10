use json::JsonValue;
use shared_lib::{error::MaelstromError, message_handler::MessageHandler};
use crate::{
    counters::pn_counter::PnCounter, states::maelstrom_node_state::MaelstromNodeState,
};

pub struct ReplicateHandler {}

impl MessageHandler for ReplicateHandler {
    type State = MaelstromNodeState;

    fn make_response_body(
        &self,
        _message: &JsonValue,
        _curr_state: &MaelstromNodeState,
    ) -> Result<JsonValue, MaelstromError> {
        unimplemented!("Replicate Handler uses get_response_body")
    }

    fn get_response_body(
        &self,
        message: &JsonValue,
        curr_state: &MaelstromNodeState,
    ) -> Result<Option<JsonValue>, MaelstromError> {
        let counters = PnCounter::from_json(&message["body"]["value"]);
        curr_state.merge_messages(counters);
        Ok(None)
    }
}
