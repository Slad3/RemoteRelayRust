use crate::models::relays::{RelayActions, RelayType};
use rocket::serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::Error;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Preset {
    pub(crate) name: String,
    pub(crate) enabled: bool,
    pub(crate) relays: HashMap<String, bool>,
}

pub(crate) fn set_preset(
    preset: &Preset,
    relays: &mut HashMap<String, RelayType>,
) -> Result<Value, Error> {
    for (relay_name, relay) in relays.iter_mut() {
        match preset.relays.get_key_value(relay_name) {
            Some((_, &value)) => {
                if value {
                    relay.turn_on().expect("Couldn't turn on");
                } else {
                    relay.turn_off().expect("Couldn't turn off");
                }
            }
            None => {
                relay.turn_off().expect("Couldn't turn off");
            }
        }
    }

    Ok(json!({"presetSet": true}))
}

pub(crate) fn get_preset_names(presets: &HashMap<String, Preset>) -> Result<Vec<Value>, Error> {
    let mut keys: Vec<String> = presets.keys().map(|key| key.clone().to_string()).collect();
    keys.sort();
    Ok(keys.into_iter().map(|s| Value::from(s)).collect())
}
