use std::fs;
use rocket::tokio::sync::Mutex;
use serde_json::{json, Value };

use serde::{Serialize, Deserialize};

use crate::relays::{KasaPlug};


#[derive(Debug, Serialize, Deserialize)]
struct Relay {
    typed: String,
    name: String,
    ip: String,
    room: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Preset{
    name: String,
    enabled: bool,
    relays: Value
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoadedConfig {
    relays: Vec<Relay>,
    // presets: Vec<Preset>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    relays: Vec<Relay>,
    // presets: Vec<Preset>
}

pub fn load_config_from_file() -> serde_json::Result<LoadedConfig> {
    let data = fs::read_to_string("config.json").expect("Unable to read file");
    serde_json::from_str::<LoadedConfig>(data.as_str())
}


fn load_relays(from_config: Vec<Relay>) -> Vec<KasaPlug> {
    let mut relays: Vec<KasaPlug> = Vec::new();

    for i in from_config{
        // let mut plug = KasaPlug::new_static("192.168.0.109", "LampLight", "office");
        if(i.typed == "KasaPlug") {
            let mut plug = KasaPlug::new(i.ip, i.name, i.room);
            let connected = plug.connected();

            match connected {
                Ok(_) => {
                    let _ = plug.get_status();
                    relays.push(plug);
                },
                Err(error) => println!("Unable to connnect {}", plug.name)
            }
        }
    }

    relays
}


fn load_presets(from_config: &Value) -> Vec<Preset> {
    let mut presets: Vec<Preset> = Vec::new();

    for (key, value) in from_config.as_object().unwrap(){
        presets.push(Preset{
            name: key.to_string(),
            enabled: value["enabled"].as_bool().unwrap(),
            relays: value["relays"].clone()
        })
    }

    presets
}

pub fn load_config() -> Vec<KasaPlug> {
    let loaded_config = load_config_from_file().unwrap();

    let relays: Vec<KasaPlug> = load_relays( loaded_config.relays);
    // let presets = load_presets(&loaded_config["presets"]);


    // Config{
    //     relays: relays,
    //     presets: presets
    // }
    relays
}

