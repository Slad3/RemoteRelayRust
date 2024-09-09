use std::collections::{HashMap, HashSet};

use crate::models::data_thread_models::{
    DataThreadCommand, DataThreadResponse, PresetCommand, RelayCommand, RelayCommands,
};
use crate::models::presets::{get_preset_names, set_preset, Preset};
use crate::models::relays::RelayType;

use crate::utils::load_config::{load_config, ConfigLocation};

use rocket::serde::json::Json;
use serde_json::{json, Value};
use std::io::Error;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Mutex;
use std::thread;
use std::thread::JoinHandle;

pub(crate) fn handle_command_input(input: &str) -> Option<RelayCommands> {
    match input.to_uppercase().as_str() {
        "TRUE" => Option::from(RelayCommands::TRUE),
        "FALSE" => Option::from(RelayCommands::FALSE),
        "SWITCH" => Option::from(RelayCommands::SWITCH),
        "STATUS" => Option::from(RelayCommands::STATUS),
        "ON" => Option::from(RelayCommands::TRUE),
        "OFF" => Option::from(RelayCommands::FALSE),
        _ => None,
    }
}

pub(crate) fn unwrap_response(package: DataThreadResponse) -> Json<Value> {
    match package {
        DataThreadResponse::Value(value) => Json(value),
        _ => Json(json!( {"Error": ""})),
    }
}

fn handle_relay_command(
    relay_command: RelayCommand,
    relays: &mut HashMap<String, RelayType>,
) -> Result<DataThreadResponse, Error> {
    if let Some(relay) = relays.get_mut(&relay_command.name) {
        match relay_command.command {
            RelayCommands::SWITCH => Ok(DataThreadResponse::Value(match relay {
                RelayType::KasaPlug(plug) => plug.switch()?,
                RelayType::KasaMultiPlug(plug) => plug.switch()?,
            })),
            RelayCommands::TRUE => Ok(DataThreadResponse::Value(match relay {
                RelayType::KasaPlug(plug) => plug.turn_on()?,
                RelayType::KasaMultiPlug(plug) => plug.turn_on()?,
            })),
            RelayCommands::FALSE => Ok(DataThreadResponse::Value(match relay {
                RelayType::KasaPlug(plug) => plug.turn_off()?,
                RelayType::KasaMultiPlug(plug) => plug.turn_off()?,
            })),
            RelayCommands::STATUS => Ok(DataThreadResponse::Bool(match relay {
                RelayType::KasaPlug(plug) => plug.get_status()?,
                RelayType::KasaMultiPlug(plug) => plug.get_status()?,
            })),
        }
    } else {
        Ok(DataThreadResponse::Bool(false))
    }
}

fn handle_preset_command(
    preset_command: PresetCommand,
    relays: &mut HashMap<String, RelayType>,
    presets: &mut HashMap<String, Preset>,
    current_preset: &Mutex<String>,
) -> Result<DataThreadResponse, Error> {
    match preset_command {
        PresetCommand::Names => match get_preset_names(presets) {
            Ok(response) => Ok(DataThreadResponse::Value(Value::Array(response))),
            Err(error) => Err(error),
        },
        PresetCommand::Set(preset_name) => {
            if let Some(preset) = presets.get_mut(&preset_name) {
                match set_preset(preset, relays) {
                    Ok(boolean) => {
                        let mut temp_current_preset = current_preset.lock().unwrap();
                        *temp_current_preset = preset.name.clone().to_string();
                        Ok(DataThreadResponse::Bool(boolean))
                    }
                    Err(error) => Err(error),
                }
            } else {
                Ok(DataThreadResponse::Bool(false))
            }
        }
    }
}

fn handle_command(
    received: DataThreadCommand,
    relays: &mut HashMap<String, RelayType>,
    presets: &mut HashMap<String, Preset>,
    current_preset: &Mutex<String>,
) -> Result<DataThreadResponse, Error> {
    match received {
        DataThreadCommand::Relay(relay_command) => handle_relay_command(relay_command, relays),
        DataThreadCommand::Preset(preset_command) => {
            handle_preset_command(preset_command, relays, presets, current_preset)
        }
        DataThreadCommand::SystemStatus => Ok(DataThreadResponse::Value(get_status(
            relays,
            current_preset.lock().unwrap().to_string(),
        )?)),
        DataThreadCommand::Refresh => Ok(DataThreadResponse::Bool(false)),
    }
}

pub(crate) fn get_status(
    relays: &HashMap<String, RelayType>,
    current_preset: String,
) -> Result<Value, Error> {
    let mut result: Value = json!({});
    let mut relay_statuses: Vec<Value> = Vec::new();
    let mut rooms: HashSet<String> = HashSet::new();

    for (_, relay) in relays.iter() {
        relay_statuses.push(match relay {
            RelayType::KasaPlug(plug) => plug.to_json(),
            RelayType::KasaMultiPlug(plug) => plug.to_json(),
        });

        rooms.insert(match relay {
            RelayType::KasaPlug(plug) => plug.room.clone(),
            RelayType::KasaMultiPlug(plug) => plug.room.clone(),
        });
    }

    result["relays"] = Value::Array(relay_statuses);
    result["rooms"] = Value::Array(rooms.into_iter().map(Value::String).collect());
    result["currentPreset"] = Value::String(current_preset.clone());

    Ok(result)
}

pub(crate) fn setup_data_thread(
    sender: Sender<DataThreadResponse>,
    receiver: Receiver<DataThreadCommand>,
    config_location: ConfigLocation,
) -> JoinHandle<()> {
    let loaded_config = load_config(config_location).expect("Could not set up thread");
    thread::spawn(move || {
        let mut relays: HashMap<String, RelayType> = loaded_config.relays;
        let mut presets: HashMap<String, Preset> = loaded_config.presets;
        let current_preset = Mutex::new("Custom".to_string());

        println!("Relays:");
        for (relay_name, _) in relays.iter() {
            println!("\t{relay_name}");
        }

        println!("Presets:");
        for (preset_name, _) in presets.iter() {
            println!("\t{preset_name}");
        }

        for received in receiver {
            match received {
                DataThreadCommand::Refresh => match load_config(config_location) {
                    Ok(config) => {
                        relays = config.relays;
                        presets = config.presets;
                        sender
                            .send(DataThreadResponse::Bool(true))
                            .expect("Channel possibly not open");
                    }
                    Err(_) => {
                        sender
                            .send(DataThreadResponse::Error(
                                "Could not refresh config".to_string(),
                            ))
                            .expect("Channel possibly not open");
                    }
                },
                _ => {
                    let response =
                        handle_command(received, &mut relays, &mut presets, &current_preset)
                            .expect("TODO: panic message");
                    sender.send(response).expect("TODO: panic message");
                }
            }
        }
    })
}
