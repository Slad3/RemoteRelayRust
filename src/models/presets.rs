use crate::models::relays::{KasaPlug, Relay};
use rocket::serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::Error;
use std::sync::Mutex;

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

pub(crate) fn set_preset(
    mut preset: &Preset,
    relays: &Mutex<HashMap<String, KasaPlug>>,
) -> Result<bool, Error> {
    let mut relays = relays.lock().expect("Error getting global RELAYS");
    for (relay_name, relay) in relays.iter_mut() {
        let rel = preset.relays.get_key_value(relay_name);
        match rel {
            Some((_, &value)) => {
                if value {
                    relay.turn_on().expect("Couldn't turn on");
                } else {
                    relay.turn_on().expect("Couldn't turn off");
                }
            }
            None => {
                relay.turn_off().expect("Couldn't find relay in preset");
            }
        }
    }
    Ok(true)
}

pub(crate) fn get_preset_names(
    presets: &Mutex<HashMap<String, Preset>>,
) -> Result<Vec<Value>, Error> {
    let presets = presets.lock().unwrap();
    let names: Vec<Value> = presets
        .keys()
        .map(|key| Value::String(key.clone().to_string()))
        .collect();
    Ok(names)
}
