use std::collections::HashMap;

use crate::models::config_models::Config;
use crate::models::presets::Preset;
use crate::models::relays::{ConfigRelay, ConfigRelayType, KasaMultiPlug, KasaPlug};
use crate::models::relays::{RelayActions, RelayType};
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct LoadedConfig {
    relays: Vec<ConfigRelay>,
    presets: Vec<Preset>,
}

pub fn load_config_from_file() -> Result<LoadedConfig, std::io::Error> {
    let data = fs::read_to_string("config.json");

    if data.is_err() {
        panic!("Couldn't find 'config.json'")
    }

    let configuration = from_str(data?.as_str())?;
    Ok(configuration)
}

fn load_relays(from_config: Vec<ConfigRelay>) -> HashMap<String, RelayType> {
    let mut relays: HashMap<String, RelayType> = HashMap::new();

    for relay in from_config {
        match relay.relay_type {
            ConfigRelayType::KasaMultiPlug => {
                let plugs = KasaMultiPlug::new(relay.ip, relay.names, relay.room);

                for mut plug in plugs {
                    if plug.connected().is_ok() {
                        relays.insert(plug.name.clone(), RelayType::KasaMultiPlug(plug));
                    }
                }
            }
            ConfigRelayType::KasaPlug => {
                let mut plug = KasaPlug::new(relay.ip, relay.name, relay.room);
                match plug.connected() {
                    Ok(_) => {
                        let _ = plug.get_status();
                        relays.insert(plug.name.clone(), RelayType::KasaPlug(plug));
                    }
                    Err(error) => {
                        rocket::log::private::error!("Unable to connnect {} {}", plug.name, error)
                    }
                }
            }
        }
    }

    relays
}

fn load_presets(from_config: Vec<Preset>) -> HashMap<String, Preset> {
    let mut presets: HashMap<String, Preset> = HashMap::new();
    for preset in from_config {
        presets.insert(preset.name.clone(), preset);
    }

    presets
        .entry("Custom".to_string())
        .or_insert_with(|| Preset {
            name: "Custom".to_string(),
            enabled: false,
            relays: HashMap::new(),
        });

    presets
        .entry("FullOff".to_string())
        .or_insert_with(|| Preset {
            name: "FullOff".to_string(),
            enabled: false,
            relays: HashMap::new(),
        });

    presets
}

pub fn load_local_config() -> Result<Config, std::io::Error> {
    let loaded_config = load_config_from_file()?;

    let relays: HashMap<String, RelayType> = load_relays(loaded_config.relays);
    let presets: HashMap<String, Preset> = load_presets(loaded_config.presets);
    Ok(Config { relays, presets })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loading_from_file_success() {
        let loaded_config = load_config_from_file();
        assert!(loaded_config.is_ok())
    }

    #[test]
    fn test_loading_relays_formatted_success() {
        let loaded_config = load_config_from_file().expect("Config File Not Found");
        let relays = load_relays(loaded_config.relays);
    }

    #[test]
    fn test_loading_presets_formatted_success() {
        let loaded_config = load_config_from_file().expect("Config File Not Found");
        let presets = load_presets(loaded_config.presets);
    }
}
