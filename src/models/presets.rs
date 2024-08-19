use crate::models::relays::KasaPlug;
use rocket::http::hyper::body::HttpBody;
use rocket::serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::io::Error;

use std::sync::Mutex;

#[derive(Debug, Serialize, Deserialize)]
pub struct Preset {
    pub(crate) name: String,
    pub(crate) enabled: bool,
    pub(crate) relays: HashMap<String, bool>,
}

pub(crate) fn set_preset(
    preset: &Preset,
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
    let mut keys: Vec<String> = presets
        .lock()
        .unwrap()
        .keys()
        .map(|key| key.clone().to_string())
        .collect();
    keys.sort();
    Ok(keys.into_iter().map(|s| Value::from(s)).collect())
}
