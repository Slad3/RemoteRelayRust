use std::collections::HashMap;
use crate::models;

use serde::{Deserialize, Serialize};
use serde_json::from_str;
use std::fs;

use models::presets::Preset;
use models::relays::{KasaPlug, Relay};

#[derive(Debug, Serialize, Deserialize)]
pub struct LoadedConfig {
    relays: Vec<Relay>,
    presets: Vec<Preset>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub(crate) relays: Vec<KasaPlug>,
    pub(crate) presets: HashMap<String, Preset>,
}

pub fn load_config_from_file() -> Result<LoadedConfig, std::io::Error> {
    let data = fs::read_to_string("config.json")?;

    let configuration = from_str(data.as_str())?;
    Ok(configuration)
}

fn load_relays(from_config: Vec<Relay>) -> Vec<KasaPlug> {
    let mut relays: Vec<KasaPlug> = Vec::new();

    for relay in from_config {
        if relay.relay_type == "KasaPlug" {
            let mut plug = KasaPlug::new(relay.ip, relay.name, relay.room);

            let connected = plug.connected();

            match connected {
                Ok(_) => {
                    let _ = plug.get_status();
                    relays.push(plug);
                }
                Err(error) => error!("Unable to connnect {} {}", plug.name, error),
            }
        }
    }

    relays
}

fn load_presets(from_config: Vec<Preset>) -> HashMap<String, Preset> {
    let mut presets: HashMap<String, Preset> = HashMap::new();
    for i in from_config {
        presets.insert(i.name.clone(), Preset {
            name: i.name,
            enabled: false,
            relays: i.relays,
        });
    }

    presets
}

pub fn load_config() -> Result<Config, std::io::Error> {
    let loaded_config = load_config_from_file()?;

    let relays: Vec<KasaPlug> = load_relays(loaded_config.relays);
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
    fn test_loading_presetes_formatted_success() {
        let loaded_config = load_config_from_file().expect("Config File Not Found");
        let presets = load_presets(loaded_config.presets);
        println!("{:?}", &presets);
    }
}
