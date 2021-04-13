use json::JsonValue;

pub trait KVValue: Clone + Default {
    fn from_json(json: &JsonValue) -> Self;

    fn to_json(&self) -> JsonValue;
}

impl KVValue for Vec<i32> {
    fn from_json(json: &JsonValue) -> Self {
        json.members().map(|jv| jv.as_i32().unwrap()).collect()
    }

    fn to_json(&self) -> JsonValue {
        let mut arr = JsonValue::new_array();
        self.iter().for_each(|i| {
            arr.push(*i);
        });
        arr
    }
}
