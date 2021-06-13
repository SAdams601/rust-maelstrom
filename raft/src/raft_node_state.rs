use shared_lib::node_state::NodeState;
use std::sync::mpsc::SyncSender;

pub(crate) struct RaftNodeState {}

impl RaftNodeState {
    pub(crate) fn set_other_node_ids(&self, p0: Vec<&str>) {
        todo!()
    }
}

impl RaftNodeState {
    pub(crate) fn set_node_id(&self, id: String) {
        todo!()
    }
}

impl NodeState for RaftNodeState {
    fn get_channel(&self) -> SyncSender<String> {
        todo!()
    }

    fn next_msg_id(&self) -> i32 {
        todo!()
    }

    fn node_id(&self) -> String {
        todo!()
    }
}
