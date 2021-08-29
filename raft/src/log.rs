use std::io::{stderr, Write};
use json::{JsonValue, object};
use std::str::FromStr;
use crate::log::Op::*;

#[derive(Copy, Clone, Debug)]
pub enum Op {
    CAS,
    Read,
    Write,
}

fn to_str(op: Op) -> &'static str {
    match op {
        CAS => "CAS",
        Read => "Read",
        Write => "Write"
    }
}

impl FromStr for Op {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "CAS" => Ok(CAS),
            "Read" => Ok(Read),
            "Write" => Ok(Write),
            _ => Err(())
        }
    }
}

impl From<Op> for JsonValue {
    fn from(op: Op) -> Self {
        JsonValue::from(to_str(op))
    }
}


#[derive(Copy, Clone, Debug)]
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
    pub fn init(node: String) -> Log {
        Log {
            node,
            entries: vec![Entry { term: 0, op: None }],
        }
    }

    pub fn get(&self, index: usize) -> Entry {
        let entry = self.entries[index - 1];
        entry.clone()
    }

    pub fn last(&self) -> Entry {
        self.entries.last().unwrap().clone()
    }

    pub fn size(&self) -> usize {
        self.entries.len()
    }

    pub fn append(&mut self, new_entries: &mut Vec<Entry>) {
        self.entries.append(new_entries);
        stderr().write(format!("{:?}\n", self.entries).as_bytes());
    }

    pub fn upto_index(&self, count: usize) -> Vec<Entry> {
        let mut vec = Vec::new();
        for i in (count-1)..self.entries.len() {
            vec.push(*self.entries.get(i).unwrap());
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
