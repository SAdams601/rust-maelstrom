use json::{object, JsonValue};
use shared_lib::{error::MaelstromError, message_handler::MessageHandler, message_utils::get_body};
use crate::{
    lin_kv_service::LinKvService,
    states::maelstrom_node_state::MaelstromState,
};

pub struct InitHandler<'a> {
    kv_service: &'a LinKvService,
}

impl InitHandler<'_> {
    pub fn init(service: &LinKvService) -> InitHandler {
        InitHandler {
            kv_service: service,
        }
    }
}

impl MessageHandler<MaelstromState> for InitHandler<'_> {

    fn make_response_body(
        &self,
        message: &json::JsonValue,
        curr_state: &MaelstromState,
    ) -> Result<JsonValue, MaelstromError> {
        let body = get_body(message);
        curr_state.set_node_id(body["node_id"].to_string());
        curr_state.set_other_node_ids(
            body["node_ids"]
                .members()
                .map(|jv| jv.as_str().unwrap())
                .collect(),
        );
        self.kv_service.init_root();
        Ok(object! {type: "init_ok"})
    }
}
