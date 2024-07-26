use rocket::serde::{Deserialize, Serialize};
use serde_json::{from_str, json, Value};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Preset {
    pub(crate) name: String,
    pub(crate) enabled: bool,
    pub(crate) relays: HashMap<String, bool>,
}

impl Preset {
    pub fn to_json(&self) -> Value {
        json!({
            "name": &self.name,
            "enabled": &self.enabled,
            "relays": &self.relays
        })
    }
}
