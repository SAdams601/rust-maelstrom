use std::io::{stderr, Write};

use json::{array, object, JsonValue};

use crate::{
    lin_kv_service::LinKvService,
    states::{datomic_state::DatomicState, serializable_map::SerializableMap},
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
        curr_state: &crate::states::node_state::NodeState,
    ) -> json::JsonValue {
        let txns = handle_txns(&message["body"]["txn"], self.kv_service);
        object! {type: "txn_ok", txn: txns}
    }
}

fn handle_txns(txns: &JsonValue, service: &LinKvService) -> JsonValue {
    let mut arr = JsonValue::new_array();
    let mut map = service.read_root();
    for txn_json in txns.members() {
        let txn = parse_txn(txn_json).unwrap();
        let txn2 = execute_txn(txn, &mut map);
        arr.push(txn2);
    }
    let res = service.cas_root(map);
    if res.is_err() {
        panic!(res.err().unwrap());
    }
    arr
}

enum TxnOp {
    Read(i32),
    Append(i32, i32),
}

fn execute_txn(txn: TxnOp, map: &mut SerializableMap) -> JsonValue {
    match txn {
        TxnOp::Read(k) => {
            let v = map.read(k);
            array!["r", k, v]
        }
        TxnOp::Append(k, v) => {
            map.append(k, v);
            array!["append", k, v]
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
