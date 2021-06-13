use json::{object, JsonValue};
use shared_lib::{error::MaelstromError, message_handler::MessageHandler};
use crate::{states::maelstrom_node_state::MaelstromState};

pub struct EchoHandler {}

impl MessageHandler<MaelstromState> for EchoHandler {

    fn make_response_body(
        &self,
        message: &JsonValue,
        _curr_state: &MaelstromState,
    ) -> Result<JsonValue, MaelstromError> {
        Ok(object! {type: "echo_ok", echo: message["body"]["echo"].clone()})
    }
}
