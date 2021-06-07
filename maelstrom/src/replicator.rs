use std::{thread, time::Duration};

use json::{object, stringify, JsonValue};

use crate::states::node_state::NodeState;

pub fn send_values(state: &'static NodeState) {
    thread::spawn(move || loop {
        thread::sleep(Duration::from_millis(5000));
        let values = state.counters_state();
        let body = object! {type: "replicate", value: values};
        let common_message = object! {body: body, src: state.node_id()};
        state.other_nodes().iter().for_each(|node_id| {
            let mut message = common_message.clone();
            message["dest"] = JsonValue::from(node_id.clone());
            state.get_channel().send(stringify(message));
        });
    });
}
