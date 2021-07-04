use std::io::{stderr, Write};

#[derive(Copy,Clone, Debug)]
pub enum Op {
    CAS,
    Read,
    Write
}

#[derive(Copy,Clone, Debug)]
pub struct Entry {
    pub term : i32,
    pub op: Option<Op>
}

pub struct Log {
    node: String,
    entries: Vec<Entry>
}

impl Log {
    pub fn init(node : String) -> Log {
        Log {
            node,
            entries: vec![Entry{ term: 0, op: None }]
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
        stderr().write(format!("{:?}", self.entries).as_bytes());
    }
}