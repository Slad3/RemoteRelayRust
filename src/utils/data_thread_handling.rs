use std::collections::{HashMap, HashSet};

use crate::models::{
    data_thread_models::{
        DataThreadCommand, DataThreadResponse, PresetCommand, RelayCommand, RelayCommands,
        TagCommand,
    },
    presets::{get_preset_names, set_preset, Preset},
    relays::{config_equals, RelayActions, RelayType},
};

use crate::utils::load_config::{load_config, ConfigLocation};

use rocket::form::validate::Contains;
use rocket::serde::json::Json;
use serde_json::{json, Value};
use std::io::{Error, ErrorKind};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

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
        DataThreadResponse::Bool(bool) => Json(Value::from(bool)),
        _ => Json(json!( {"Error": "Unrecognized response type."})),
    }
}

fn handle_relay_command(
    relay_command: RelayCommand,
    relays: &mut HashMap<String, RelayType>,
    current_preset: &Mutex<String>,
) -> Result<DataThreadResponse, Error> {
    if let Some(relay) = relays.get_mut(&relay_command.name) {
        match relay_command.command {
            RelayCommands::SWITCH | RelayCommands::TRUE | RelayCommands::FALSE => {
                let mut temp_current_preset = current_preset.lock().unwrap();
                *temp_current_preset = "Custom".to_string()
            }
            _ => {}
        }

        match relay_command.command {
            RelayCommands::SWITCH => Ok(DataThreadResponse::Value(relay.switch()?)),
            RelayCommands::TRUE => Ok(DataThreadResponse::Value(relay.turn_on()?)),
            RelayCommands::FALSE => Ok(DataThreadResponse::Value(relay.turn_off()?)),
            RelayCommands::STATUS => Ok(DataThreadResponse::Value(
                json!({"status": relay.get_status()?}),
            )),
        }
    } else {
        Ok(DataThreadResponse::Bool(false))
    }
}

fn handle_tag_command(
    tag_command: TagCommand,
    relays: &mut HashMap<String, RelayType>,
    current_preset: &Mutex<String>,
) -> Result<DataThreadResponse, Error> {
    match tag_command.command {
        RelayCommands::SWITCH | RelayCommands::TRUE | RelayCommands::FALSE => {
            let mut temp_current_preset = current_preset.lock().unwrap();
            *temp_current_preset = "Custom".to_string()
        }
        _ => {}
    }

    let mut statuses: Vec<Value> = Vec::new();

    let mut found = false;
    for relay in relays.values_mut().filter(|relay| {
        let tags = match relay {
            RelayType::KasaPlug(ref relay) => &relay.tags,
            RelayType::KasaMultiPlug(ref relay) => &relay.tags,
        };
        tags.contains(&tag_command.tag)
    }) {
        found = true;

        let _ = match tag_command.command {
            RelayCommands::SWITCH => {
                let _ = relay.switch()?;
            }
            RelayCommands::TRUE => {
                let _ = relay.turn_on()?;
            }
            RelayCommands::FALSE => {
                let _ = relay.turn_off()?;
            }
            RelayCommands::STATUS => statuses.push(json!({"status": relay.to_json()})),
        };
    }

    if !found {
        return Err(Error::new(
            ErrorKind::NotFound,
            format!("No relays with tag: {} found", &tag_command.tag),
        ));
    }

    match tag_command.command {
        RelayCommands::STATUS => Ok(DataThreadResponse::Value(Value::from(statuses))),
        _ => Ok(DataThreadResponse::Bool(true)),
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
        PresetCommand::Set(preset_name) => match presets.get_mut(&preset_name) {
            Some(preset) => match set_preset(preset, relays) {
                Ok(value) => {
                    let mut temp_current_preset = current_preset.lock().unwrap();
                    *temp_current_preset = preset.name.clone().to_string();
                    Ok(DataThreadResponse::Value(value))
                }
                Err(error) => Err(error),
            },
            None => Err(Error::new(ErrorKind::NotFound, "Unable to find preset")),
        },
    }
}

fn handle_command(
    received: DataThreadCommand,
    relays: &mut HashMap<String, RelayType>,
    presets: &mut HashMap<String, Preset>,
    current_preset: &Mutex<String>,
) -> Result<DataThreadResponse, Error> {
    match received {
        DataThreadCommand::Relay(relay_command) => {
            handle_relay_command(relay_command, relays, current_preset)
        }
        DataThreadCommand::Preset(preset_command) => {
            handle_preset_command(preset_command, relays, presets, current_preset)
        }
        DataThreadCommand::SystemStatus => Ok(DataThreadResponse::Value(get_status(
            relays,
            current_preset.lock().unwrap().to_string(),
        )?)),
        DataThreadCommand::Tag(tag_command) => {
            handle_tag_command(tag_command, relays, current_preset)
        }
        DataThreadCommand::Refresh => Ok(DataThreadResponse::Bool(false)),
        DataThreadCommand::AutoRefresh => Ok(DataThreadResponse::Bool(false)),
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
        relay_statuses.push(relay.to_json());

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

#[allow(unused)]
fn setup_update_thread(
    route_to_data_sender: Sender<DataThreadCommand>,
    refresh_time: u64,
) -> JoinHandle<bool> {
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(refresh_time));
        if route_to_data_sender
            .send(DataThreadCommand::AutoRefresh)
            .is_err()
        {
            eprintln!("Unable to send refresh command");
        }
    })
}

pub(crate) fn setup_data_thread(
    sender: Sender<DataThreadResponse>,
    receiver: Receiver<DataThreadCommand>,
    route_to_data_sender: Sender<DataThreadCommand>,
    config_location: ConfigLocation,
) -> JoinHandle<()> {
    let loaded_config = load_config(config_location)
        .join()
        .expect("Could not set up thread")
        .unwrap();

    thread::spawn(move || {
        let relays = Arc::new(Mutex::new(loaded_config.relays));
        let presets = Arc::new(Mutex::new(loaded_config.presets));
        let current_preset = Arc::new(Mutex::new("Custom".to_string()));

        setup_update_thread(route_to_data_sender.clone(), 10);

        for received in receiver {
            match received {
                DataThreadCommand::Refresh | DataThreadCommand::AutoRefresh => {
                    match load_config(config_location)
                        .join()
                        .expect("Unable to join config thread")
                    {
                        Ok(config) => {
                            let mut relays = relays.lock().expect("Failed to lock relays");
                            let mut presets = presets.lock().expect("Failed to lock presets");

                            if !config_equals::<RelayType>(&*relays, &config.relays) {
                                *relays = config.relays;
                            }

                            if !config_equals::<Preset>(&*presets, &config.presets) {
                                *presets = config.presets;
                            }

                            match received {
                                DataThreadCommand::Refresh => sender
                                    .send(DataThreadResponse::Bool(true))
                                    .expect("Channel possibly not open"),
                                _ => {}
                            }
                        }
                        Err(_) => {
                            sender
                                .send(DataThreadResponse::Error(
                                    "Could not refresh config".to_string(),
                                ))
                                .expect("Channel possibly not open");
                        }
                    }
                }
                _ => {
                    let mut relays = relays.lock().expect("Failed to lock relays");
                    let mut presets = presets.lock().expect("Failed to lock presets");

                    let response =
                        handle_command(received, &mut *relays, &mut *presets, &current_preset)
                            .unwrap_or_else(|error| DataThreadResponse::Error(error.to_string()));

                    match response {
                        DataThreadResponse::Error(error) => {
                            eprintln!("Error sending command: {:?}", &error);
                        }
                        _ => sender.send(response).expect("Channel possibly not open"),
                    }
                }
            }
        }
    })
}
