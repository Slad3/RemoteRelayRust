use crate::models::relays::RelayType;
use rocket::serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::io::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct Preset {
    pub(crate) name: String,
    pub(crate) enabled: bool,
    pub(crate) relays: HashMap<String, bool>,
}

pub(crate) fn set_preset(
    preset: &Preset,
    relays: &mut HashMap<String, RelayType>,
) -> Result<bool, Error> {
    for (relay_name, relay) in relays.iter_mut() {
        match preset.relays.get_key_value(relay_name) {
            Some((_, &value)) => {
                if value {
                    match relay {
                        RelayType::KasaPlug(plug) => plug.turn_on(),
                        RelayType::KasaMultiPlug(plug) => plug.turn_on(),
                    }
                    .expect("Couldn't turn on");
                } else {
                    match relay {
                        RelayType::KasaPlug(plug) => plug.turn_off(),
                        RelayType::KasaMultiPlug(plug) => plug.turn_off(),
                    }
                    .expect("Couldn't turn off");
                }
            }
            None => {
                match relay {
                    RelayType::KasaPlug(plug) => plug.turn_off(),
                    RelayType::KasaMultiPlug(plug) => plug.turn_off(),
                }
                .expect("Couldn't turn off");
            }
        }
    }

    Ok(true)
}

pub(crate) fn get_preset_names(presets: &HashMap<String, Preset>) -> Result<Vec<Value>, Error> {
    let mut keys: Vec<String> = presets.keys().map(|key| key.clone().to_string()).collect();
    keys.sort();
    Ok(keys.into_iter().map(|s| Value::from(s)).collect())
}
