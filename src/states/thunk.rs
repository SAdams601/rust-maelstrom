use std::{
    io::{stderr, Write},
    sync::RwLock,
};

use crate::{error::DefiniteError, lin_kv_service::LinKvService};

pub struct Thunk {
    pub id: String,
    value: RwLock<Option<Vec<i32>>>,
    pub saved: RwLock<bool>,
}

impl Thunk {
    pub fn init(id: String, v: Option<Vec<i32>>, saved: bool) -> Thunk {
        Thunk {
            id: id,
            value: RwLock::new(v),
            saved: RwLock::new(saved),
        }
    }

    pub fn value(&self, service: &LinKvService) -> Vec<i32> {
        let m_val = self.value.read().unwrap();
        if m_val.is_some() {
            return m_val.as_ref().unwrap().clone();
        }
        drop(m_val);
        let vec = service.read_thunk_list(&self.id);
        let mut thunk_val = self.value.write().unwrap();
        *thunk_val = Some(vec.clone());
        vec
    }

    pub fn save(&self, service: &LinKvService) -> Result<(), DefiniteError> {
        if *self.saved.read().unwrap() {
            return Ok(());
        }
        let result = service.save_thunk(self)["body"]["type"].to_string();
        if result != "write_ok" {
            return Err(crate::error::abort(format!(
                "Failed to save thunk with id {}",
                self.id
            )));
        }
        let mut svd = self.saved.write().unwrap();
        *svd = true;
        Ok(())
    }
}
