use std::io::{stderr, Write};

use json::{array, object, JsonValue};

use crate::{
    error::{DefiniteError, MaelstromError},
    lin_kv_service::LinKvService,
    states::serializable_map::SerializableMap,
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
        _curr_state: &crate::states::node_state::NodeState,
    ) -> Result<JsonValue, MaelstromError> {
        let txns = self.handle_txns(&message["body"]["txn"]);
        txns.map(|txn| object! {type: "txn_ok", txn: txn})
            .map_err(|s| self.make_error(message, s))
    }
}

impl TxnHandler<'_> {
    fn handle_txns(&self, txns: &JsonValue) -> Result<JsonValue, DefiniteError> {
        let mut arr = JsonValue::new_array();
        let mut map = self.kv_service.read_root();
        for txn_json in txns.members() {
            let txn = parse_txn(txn_json).unwrap();
            let txn2 = self.execute_txn(txn, &mut map);
            arr.push(txn2);
        }
        let save_res = map.save_thunks(self.kv_service);
        if save_res.is_err() {
            return Err(save_res.err().unwrap());
        }
        let cas_res = self.kv_service.cas_root(map);
        cas_res.map(|_| arr)
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
