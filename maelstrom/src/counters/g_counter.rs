use std::collections::HashMap;

use json::{object::Object, JsonValue};

pub struct GCounter {
    values: HashMap<String, i32>,
}

impl GCounter {
    pub fn init() -> GCounter {
        GCounter {
            values: HashMap::new(),
        }
    }

    pub fn to_json(&self) -> JsonValue {
        let mut object = Object::new();
        self.values.iter().for_each(|(k, v)| {
            object.insert(k, JsonValue::from(*v));
        });
        JsonValue::Object(object)
    }

    pub fn from_json(jv: &JsonValue) -> GCounter {
        let mut map: HashMap<String, i32> = HashMap::new();
        jv.entries().for_each(|(k, v)| {
            map.insert(String::from(k), v.as_i32().unwrap());
        });
        GCounter { values: map }
    }

    pub fn read(&self) -> i32 {
        self.values.iter().fold(0, |sum, (_, v)| sum + v)
    }

    pub fn merge(&mut self, other: GCounter) {
        other.values.iter().for_each(|(k, v1)| {
            let my_val = self.values.get(k);
            match my_val {
                Some(v2) => {
                    if v1 > v2 {
                        self.values.insert(k.to_string(), *v1);
                    }
                }
                None => {
                    self.values.insert(k.to_string(), *v1);
                }
            }
        });
    }

    pub fn add(&mut self, node_id: String, delta: i32) {
        let value = self.values.get(&node_id).get_or_insert(&0).clone();
        self.values.insert(node_id, value + delta);
    }
}
