use shared_lib::node_state::NodeState;
use std::sync::mpsc::SyncSender;
use std::sync::{RwLock, Mutex};
use std::cell::RefCell;
use std::ops::Deref;

pub(crate) struct RaftState {
    node_state : NodeState
}

impl RaftState {
    pub fn init(response_channel: SyncSender<String>) -> RaftState {
        RaftState {
            node_state: NodeState::init(response_channel)
        }
    }
}

impl Deref for RaftState {
    type Target = NodeState;

    fn deref(&self) -> &Self::Target {
        &self.node_state
    }
}
