use std::collections::HashMap;

use json::JsonValue;

use crate::{error::DefiniteError, lin_kv_service::LinKvService};

use super::thunk::Thunk;

pub struct SerializableMap {
    map: HashMap<i32, Thunk>,
    changes: HashMap<i32, Thunk>,
}

impl SerializableMap {
    pub fn init(service: &LinKvService) -> SerializableMap {
        SerializableMap {
            map: HashMap::new(),
            changes: HashMap::new(),
        }
    }

    pub fn from_json(json: &JsonValue) -> SerializableMap {
        let mut map = HashMap::new();
        for (k_str, jv) in json.entries() {
            let key = k_str.parse().unwrap();
            let id = jv.to_string();
            map.insert(key, Thunk::init(id, None, true));
        }
        SerializableMap {
            map,
            changes: HashMap::new(),
        }
    }

    pub fn to_json(&self) -> JsonValue {
        let mut jv = JsonValue::new_object();
        for (k, thunk) in self.changes.iter() {
            jv.insert(&k.to_string(), thunk.id.clone());
        }
        for (k, thunk) in self.map.iter() {
            if !self.changes.contains_key(k) {
                jv.insert(&k.to_string(), thunk.id.clone());
            }
        }

        jv
    }

    pub fn original_to_json(&self) -> JsonValue {
        let mut jv = JsonValue::new_object();
        for (k, thunk) in self.map.iter() {
            jv.insert(&k.to_string(), thunk.id.clone());
        }
        jv
    }

    pub fn read(&self, k: i32, service: &LinKvService) -> Option<Vec<i32>> {
        if self.changes.contains_key(&k) {
            return self.changes.get(&k).map(|thunk| thunk.value(service));
        }
        self.map.get(&k).map(|thunk| thunk.value(service))
    }

    pub fn append(&mut self, service: &LinKvService, k: i32, v: i32) {
        let mut vec = self.read(k, service).unwrap_or(Vec::new());
        vec.push(v);
        let new_id = service.new_id();
        let thunk = Thunk::init(new_id, Some(vec), false);
        self.changes.insert(k, thunk);
    }

    pub fn save_thunks(&self, service: &LinKvService) -> Result<(), DefiniteError> {
        for (_key, thunk) in &self.changes {
            let save_res = thunk.save(service);
            if save_res.is_err() {
                return save_res;
            }
        }
        Ok(())
    }
}
