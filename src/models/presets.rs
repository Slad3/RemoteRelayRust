use std::collections::HashMap;
use rocket::serde::{Deserialize, Serialize};
use serde_json::{json, Value, from_str};

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

