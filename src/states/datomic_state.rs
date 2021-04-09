use std::{collections::HashMap, sync::RwLock};

pub struct DatomicState {
    state: HashMap<i32, Vec<i32>>,
}

impl DatomicState {
    pub fn init() -> DatomicState {
        DatomicState {
            state: HashMap::new(),
        }
    }

    pub fn read(&self, key: i32) -> Vec<i32> {
        self.state.get(&key).map_or(Vec::new(), |vec| vec.clone())
    }

    pub fn append(&mut self, key: i32, val: i32) {
        if !self.state.contains_key(&key) {
            self.state.insert(key, Vec::new());
        }
        let vec = self.state.get_mut(&key).unwrap();
        vec.push(val);
    }
}
