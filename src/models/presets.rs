use rocket::serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Mutex;
use crate::models::relays::{KasaPlug, Relay};

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

pub(crate) fn set_preset(preset_name: &String,
                         presets: &Mutex<HashMap<String, Preset>>,
                         relays: &Mutex<HashMap<String, KasaPlug>>,
) {
    let mut binding = presets.lock().unwrap();
    let preset = binding.get_mut(preset_name);
    match preset {
        Some(preset) => {
            let mut relays = relays.lock().expect("Error getting global RELAYS");
            for (relay_name, relay) in relays.iter_mut() {
                let rel = preset.relays.get_key_value(relay_name);
                match rel {
                    Some(temp) => {
                        let (_, &value) = temp;
                        if value {
                            relay.turn_on();
                        } else {
                            relay.turn_on();
                        }
                    }
                    None => {
                        println!("Not found relay {}", relay.name);
                        relay.turn_off();
                    }
                }
            }
        }
        None => {
            println!("asdfasdfas")
        }
    }
}

