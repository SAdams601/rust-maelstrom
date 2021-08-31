use json::{JsonValue, object};
use std::str::FromStr;
use crate::log::Op::*;

#[derive(Clone, Debug)]
pub enum Op {
    CAS {key: i32, from: i32, to: i32, requester: String, msg_id: i32 },
    Read {key: i32, requester: String, msg_id: i32},
    Write{key: i32, value: i32, requester: String, msg_id: i32 },
}

fn to_str(op: Op) -> String {
    match op {
        CAS {key, from, to, requester, msg_id } => format!("CAS:{},{},{},{},{}", key, from, to, requester,msg_id),
        Read {key, requester, msg_id} => format!("Read:{},{},{}", key,requester,msg_id),
        Write{key, value, requester, msg_id} => format!("Write:{},{},{},{}",key,value, requester, msg_id)
    }
}

impl FromStr for Op {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let owned = s.clone();
        let split: Vec<&str> = owned.split(":").collect();
        let op = split[0];
        let args: Vec<&str> = split[1].split(",").collect();
        match op {
            "CAS" => Ok(CAS{key: unsafe_parse_i32(args[0]),from: unsafe_parse_i32(args[1]),to: unsafe_parse_i32(args[2]), requester: args[3].to_string(), msg_id: unsafe_parse_i32(args[4])}),
            "Read" => Ok(Read{key: unsafe_parse_i32(args[0]), requester: args[1].to_string(), msg_id: unsafe_parse_i32(args[2]) }),
            "Write" => Ok(Write{key: unsafe_parse_i32(args[0]), value: unsafe_parse_i32(args[1]), requester: args[2].to_string(), msg_id: unsafe_parse_i32(args[3])}),
            _ => Err(())
        }
    }
}

fn unsafe_parse_i32(s: &str) -> i32 {
    s.parse().unwrap()
}

impl From<Op> for JsonValue {
    fn from(op: Op) -> Self {
        JsonValue::from(to_str(op))
    }
}


#[derive(Clone, Debug)]
pub struct Entry {
    pub term: i32,
    pub op: Option<Op>,
}

#[derive(Debug)]
pub struct Log {
    node: String,
    entries: Vec<Entry>,
}

impl Log {
    pub fn init<'a>(node: String) -> Log {
        Log {
            node,
            entries: vec![Entry { term: 0, op: None }],
        }
    }

    pub fn get(&self, index: usize) -> Option<Entry> {
        if self.entries.len() <= index - 1 || index <= 0 {
            return None;
        }
        Some(self.entries[index - 1].clone())
    }

    pub fn last(&self) -> Entry {
        self.entries.last().unwrap().clone()
    }

    pub fn size(&self) -> usize {
        self.entries.len()
    }

    pub fn append(&mut self, new_entries: & mut Vec<Entry>) {
        self.entries.append(new_entries);
    }

    pub fn upto_index(&self, count: usize) -> Vec<Entry> {
        let mut vec = Vec::new();
        if count > 0 {
            for i in (count - 1)..self.entries.len() {
                vec.push(self.entries.get(i).unwrap().clone());
            }
        }
        vec
    }

    pub fn truncate(&mut self, len: usize) {
        self.entries.truncate(len);
    }
}

impl From<Entry> for JsonValue {
    fn from(entry: Entry) -> Self {
        object!("op": entry.op, "term": entry.term)
    }
}

impl From<&JsonValue> for Entry {
    fn from(jv: &JsonValue) -> Self {
        Entry {
            term: jv["term"].as_i32().unwrap_or(-1),
            op: jv["op"].as_str().and_then(|s| Op::from_str(s).ok()),
        }
    }
}


pub fn parse_entries(jv: &JsonValue) -> Vec<Entry> {
    let mut v = vec![];
    for entry_jv in jv.members() {
        let entry = Entry::from(entry_jv);
        v.push(entry);
    }
    return v;
}
