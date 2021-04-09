use std::io::{stderr, Write};

use json::{array, object, JsonValue};

use crate::states::datomic_state::DatomicState;

use super::MessageHandler;

pub struct TxnHandler {}

impl MessageHandler for TxnHandler {
    fn make_response_body(
        &self,
        message: &json::JsonValue,
        curr_state: &crate::states::node_state::NodeState,
    ) -> json::JsonValue {
        let mut datomic = curr_state.borrow_datomic();
        let txns = handle_txns(&message["body"]["txn"], &mut datomic);
        object! {type: "txn_ok", txn: txns}
    }
}

fn handle_txns(txns: &JsonValue, state: &mut DatomicState) -> JsonValue {
    let mut arr = JsonValue::new_array();
    for txn_json in txns.members() {
        let txn = parse_txn(txn_json).unwrap();
        let txn2 = execute_txn(txn, state);
        arr.push(txn2);
    }
    arr
}

enum TxnOp {
    Read(i32),
    Append(i32, i32),
}

fn execute_txn(txn: TxnOp, state: &mut DatomicState) -> JsonValue {
    match txn {
        TxnOp::Read(k) => {
            let v = state.read(k);
            array!["r", k, v]
        }
        TxnOp::Append(k, v) => {
            state.append(k, v);
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
