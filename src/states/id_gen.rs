use std::sync::Mutex;

pub struct IdGenerator {
    node_id: String,
    i: Mutex<i32>,
}

impl IdGenerator {
    pub fn init(node_id: String) -> IdGenerator {
        IdGenerator {
            node_id: node_id,
            i: Mutex::new(0),
        }
    }

    pub fn get_next_id(&self) -> String {
        let mut curr_i = self.i.lock().unwrap();
        let id = format!("{}-{}", self.node_id.clone(), *curr_i);
        *curr_i += 1;
        id
    }
}
