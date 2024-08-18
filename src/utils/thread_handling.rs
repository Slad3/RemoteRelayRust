use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::models::presets::{get_preset_names, set_preset, Preset};
use crate::models::relays::KasaPlug;
use crate::utils::local_config_utils::{load_config, Config};
use rocket::response::content::RawJson;
use serde_json::{json, Value};
use std::io::Error;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Mutex;
use std::thread;
use std::thread::JoinHandle;

#[derive(Debug, Serialize, Deserialize)]
// pub(crate) enum ThreadResponse {
pub(crate) enum ThreadPackage {
    ThreadCommand(ThreadCommand),
    Response(ThreadResponse),
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum ThreadResponse {
    Value(Value),
    Bool(bool),
    Error(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum ThreadCommand {
    Status,
    Refresh,
    Relay(RelayCommand),
    Preset(PresetCommand),
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct RelayCommand {
    pub(crate) name: String,
    pub(crate) command: RelayCommands,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum PresetCommand {
    Set(String),
    Names,
    // CurrentPreset,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum RelayCommands {
    #[serde(rename = "true")]
    TRUE,
    FALSE,
    SWITCH,
    STATUS,
}

pub(crate) fn handle_command_input(input: String) -> Option<RelayCommands> {
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

pub(crate) fn unwrap_response(package: ThreadPackage) -> RawJson<String> {
    match package {
        ThreadPackage::Response(ref response) => {
            RawJson(json!( {"RelaySet": response}).to_string())
        }
        _ => RawJson(json!( {"Error": ""}).to_string()),
    }
}

fn handle_command(
    received: ThreadPackage,
    relays: &Mutex<HashMap<String, KasaPlug>>,
    presets: &Mutex<HashMap<String, Preset>>,
) -> Result<ThreadResponse, Error> {
    match received {
        ThreadPackage::ThreadCommand(command) => match command {
            ThreadCommand::Relay(relay_command) => {
                if let Some(relay) = relays.lock().unwrap().get_mut(&relay_command.name) {
                    match relay_command.command {
                        RelayCommands::SWITCH => Ok(ThreadResponse::Bool(relay.switch()?)),
                        RelayCommands::TRUE => Ok(ThreadResponse::Bool(relay.turn_on()?)),
                        RelayCommands::FALSE => Ok(ThreadResponse::Bool(relay.turn_off()?)),
                        RelayCommands::STATUS => Ok(ThreadResponse::Bool(relay.get_status()?)),
                    }
                } else {
                    Ok(ThreadResponse::Bool(false))
                }
            }
            ThreadCommand::Preset(preset_command) => match preset_command {
                PresetCommand::Names => match get_preset_names(presets) {
                    Ok(response) => Ok(ThreadResponse::Value(Value::Array(response))),
                    Err(error) => Err(error),
                },
                PresetCommand::Set(preset_name) => {
                    if let Some(preset) = presets.lock().unwrap().get_mut(&preset_name) {
                        match set_preset(preset, relays) {
                            Ok(boolean) => Ok(ThreadResponse::Bool(boolean)),
                            Err(error) => Err(error),
                        }
                    } else {
                        Ok(ThreadResponse::Bool(false))
                    }
                }
            },
            ThreadCommand::Status => Ok(ThreadResponse::Value(get_status(relays)?)),
            ThreadCommand::Refresh => Ok(ThreadResponse::Bool(false)),
        },
        _ => Ok(ThreadResponse::Bool(true)),
    }
}

pub(crate) fn get_status(relays: &Mutex<HashMap<String, KasaPlug>>) -> Result<Value, Error> {
    let mut result: Value = json!({});
    let mut relay_statuses: Vec<Value> = Vec::new();
    let mut rooms: HashSet<String> = HashSet::new();

    for (_, relay) in relays.lock().expect("Error getting global RELAYS").iter() {
        relay_statuses.push(relay.to_json());
        rooms.insert(relay.room.clone());
    }

    result["relays"] = Value::Array(relay_statuses);
    result["rooms"] = Value::Array(rooms.into_iter().map(Value::String).collect());

    Ok(result)
}

pub(crate) async fn setup_data_thread(
    sender: Sender<ThreadPackage>,
    receiver: Receiver<ThreadPackage>,
) -> JoinHandle<()> {
    let loaded_config = load_config().expect("Could not set up thread");
    thread::spawn(move || {
        let mut relays: Mutex<HashMap<String, KasaPlug>> = Mutex::new(loaded_config.relays);
        let mut presets: Mutex<HashMap<String, Preset>> = Mutex::new(loaded_config.presets);

        println!("Relays:");
        for (relay_name, _) in relays.lock().unwrap().iter() {
            println!("\t{relay_name}");
        }

        println!("Presets:");
        for (preset_name, _) in presets.lock().unwrap().iter() {
            println!("\t{preset_name}");
        }

        for received in receiver {
            println!("{:?}", &received);
            match received {
                ThreadPackage::ThreadCommand(ThreadCommand::Refresh) => {
                    let refresh_loaded_config = load_config();
                    match refresh_loaded_config {
                        Ok(config) => {
                            relays = Mutex::new(config.relays);
                            presets = Mutex::new(config.presets);
                            sender
                                .send(ThreadPackage::Response(ThreadResponse::Bool(true)))
                                .expect("Channel possibly not open");
                        }
                        Err(_) => {
                            sender
                                .send(ThreadPackage::Response(ThreadResponse::Bool(false)))
                                .expect("Channel possibly not open");
                        }
                    }
                }
                _ => {
                    let response =
                        handle_command(received, &relays, &presets).expect("TODO: panic message");
                    sender
                        .send(ThreadPackage::Response(response))
                        .expect("TODO: panic message");
                }
            }
        }
    })
}
