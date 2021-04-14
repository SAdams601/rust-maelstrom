use std::io::{stderr, Write};

use json::{array, object, JsonValue};

use crate::{
    error::{DefiniteError, MaelstromError},
    lin_kv_service::LinKvService,
    states::{node_state::NodeState, serializable_map::SerializableMap, thunk::Thunk},
};

use super::MessageHandler;

pub struct TxnHandler<'a> {
    kv_service: &'a LinKvService,
}

impl TxnHandler<'_> {
    pub fn init(service: &LinKvService) -> TxnHandler {
        TxnHandler {
            kv_service: service,
        }
    }
}

impl MessageHandler for TxnHandler<'_> {
    fn make_response_body(
        &self,
        message: &json::JsonValue,
        curr_state: &NodeState,
    ) -> Result<JsonValue, MaelstromError> {
        let txns = self.handle_txns(curr_state, &message["body"]["txn"]);
        txns.map(|txn| object! {type: "txn_ok", txn: txn})
            .map_err(|s| self.make_error(message, s))
    }
}

impl TxnHandler<'_> {
    fn handle_txns(
        &self,
        curr_state: &NodeState,
        txns: &JsonValue,
    ) -> Result<JsonValue, DefiniteError> {
        let mut arr = JsonValue::new_array();
        let thunk = self.kv_service.read_root();
        let mut map = thunk.value(self.kv_service);
        for txn_json in txns.members() {
            let txn = parse_txn(txn_json).unwrap();
            let txn2 = self.execute_txn(txn, &mut map);
            arr.push(txn2);
        }
        let save_res = map.save_thunks(self.kv_service);
        if save_res.is_err() {
            return Err(save_res.err().unwrap());
        }
        if map.has_changed() {
            let new_thunk = Thunk::init(curr_state.next_thunk_id(), Some(map), false);
            let save_res = new_thunk.save(self.kv_service);
            if save_res.is_err() {
                return Err(save_res.err().unwrap());
            }
            let cas_res = self.kv_service.cas_root(thunk.id, new_thunk.id);
            if cas_res.is_err() {
                return Err(cas_res.err().unwrap());
            }
        }
        Ok(arr)
    }

    fn make_error(&self, message: &JsonValue, error: DefiniteError) -> MaelstromError {
        MaelstromError {
            in_reply_to: (message["body"]["msg_id"].as_i32().unwrap()),
            error: error,
        }
    }

    fn execute_txn(&self, txn: TxnOp, map: &mut SerializableMap) -> JsonValue {
        match txn {
            TxnOp::Read(k) => {
                let v = map.read(k, self.kv_service);
                array!["r", k, v]
            }
            TxnOp::Append(k, v) => {
                map.append(self.kv_service, k, v);
                array!["append", k, v]
            }
        }
    }
}

fn parse_txn(txn: &JsonValue) -> Option<TxnOp> {
    let op = txn[0].as_str().unwrap();
    match op {
        "r" => Some(TxnOp::Read(txn[1].as_i32().unwrap())),
        "append" => Some(TxnOp::Append(
            txn[1].as_i32().unwrap(),
            txn[2].as_i32().unwrap(),
        )),
        _ => {
            stderr().write_all(format!("Received unknown transaction {:?}\n", txn).as_bytes());
            return None;
        }
    }
}

#[derive(Debug)]
enum TxnOp {
    Read(i32),
    Append(i32, i32),
}
