use std::sync::mpsc::SyncSender;

pub trait NodeState {
    fn get_channel(&self) -> SyncSender<String>;

    fn next_msg_id(&self) -> i32;

    fn node_id(&self) -> String;
}
