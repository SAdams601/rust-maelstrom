use std::collections::HashMap;

use json::JsonValue;

pub struct SerializableMap {
    map: HashMap<i32, Vec<i32>>,
    changes: HashMap<i32, Vec<i32>>,
}

impl SerializableMap {
    pub fn init() -> SerializableMap {
        SerializableMap {
            map: HashMap::new(),
            changes: HashMap::new(),
        }
    }

    pub fn from_json(json: &JsonValue) -> SerializableMap {
        let mut map = HashMap::new();
        for (k_str, jv) in json.entries() {
            let key = k_str.parse().unwrap();
            let value = jv.members().map(|i| i.as_i32().unwrap()).collect();
            map.insert(key, value);
        }
        SerializableMap {
            map,
            changes: HashMap::new(),
        }
    }

    pub fn to_json(&self) -> JsonValue {
        let mut jv = JsonValue::new_object();
        for (k, v) in self.changes.iter() {
            jv.insert(&k.to_string(), v.clone());
        }
        for (k, v) in self.map.iter() {
            if !self.changes.contains_key(k) {
                jv.insert(&k.to_string(), v.clone());
            }
        }

        jv
    }

    pub fn original_to_json(&self) -> JsonValue {
        let mut jv = JsonValue::new_object();
        for (k, v) in self.map.iter() {
            jv.insert(&k.to_string(), v.clone());
        }
        jv
    }

    pub fn read(&self, k: i32) -> Option<Vec<i32>> {
        if self.changes.contains_key(&k) {
            return self.changes.get(&k).cloned();
        }
        self.map.get(&k).cloned()
    }

    pub fn append(&mut self, k: i32, v: i32) {
        let mut vec = self.read(k).unwrap_or(Vec::new());
        vec.push(v);
        self.insert(k, vec);
    }

    pub fn insert(&mut self, k: i32, v: Vec<i32>) {
        self.changes.insert(k, v);
    }
}
