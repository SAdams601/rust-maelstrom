use std::{
    io::{stderr, Write},
    sync::RwLock,
};

use json::JsonValue;

use crate::{error::DefiniteError, lin_kv_service::LinKvService};

use super::kv_thunk::KVValue;

#[derive(Debug)]
pub struct Thunk<T: KVValue> {
    pub id: String,
    value: RwLock<Option<T>>,
    pub saved: RwLock<bool>,
}

impl<T: KVValue> Clone for Thunk<T> {
    fn clone(&self) -> Self {
        let is_saved = *self.saved.read().unwrap();
        Thunk::init(self.id.clone(), None, is_saved.clone())
    }
}

impl<T: KVValue> Thunk<T> {
    pub fn init(id: String, v: Option<T>, saved: bool) -> Thunk<T> {
        Thunk {
            id,
            value: RwLock::new(v),
            saved: RwLock::new(saved),
        }
    }

    pub fn value(&self, service: &LinKvService) -> T {
        let m_val = self.value.read().unwrap();
        if m_val.is_some() {
            return m_val.as_ref().unwrap().clone();
        }
        drop(m_val);
        let json = service.read_thunk_json(&self);
        let val = T::from_json(&json);
        let mut thunk_val = self.value.write().unwrap();
        *thunk_val = Some(val.clone());
        val
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
        let mut saved = self.saved.write().unwrap();
        *saved = true;
        Ok(())
    }
}
