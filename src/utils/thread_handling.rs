use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::models::presets::{get_preset_names, set_preset, Preset};
use crate::models::relays::KasaPlug;
use crate::utils::local_config_utils::load_config;
use rocket::outcome::Outcome::Error;
use rocket::response::content::RawJson;
use serde_json::{json, Value};
use std::io::Error;
use std::sync::mpsc::{Receiver, SendError, Sender};
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
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum ThreadCommand {
    Status,
    Switch,
    AllOff,
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
    CurrentPreset,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum RelayCommands {
    #[serde(rename = "true")]
    TRUE,
    FALSE,
    SWITCH,
    STATUS,
}

pub(crate) fn handle_send(result: Result<(), SendError<ThreadPackage>>) {
    match result {
        Err(error) => {
            println!("{}", error.to_string())
        }
        _ => {}
    }
}

pub(crate) fn handle_received(thread_name: &str, received: ThreadPackage) {
    match received {
        ThreadPackage::ThreadCommand(rec) => {
            println!("{thread_name}:\t{:?}", rec);
        }
        _ => {}
    }
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
    println!("{:?}", received);
    match received {
        ThreadPackage::ThreadCommand(command) => match command {
            ThreadCommand::Relay(relay_command) => {
                if let Some(mut relay) = relays.lock().unwrap().get_mut(&relay_command.name) {
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
                _ => Err(Error("Command not implemented yet")),
            },
            ThreadCommand::Status => Ok(ThreadResponse::Value(get_status(relays)?)),
            ThreadCommand::Switch => Ok(ThreadResponse::Bool(true)),
            ThreadCommand::AllOff => Ok(ThreadResponse::Bool(true)),
        },
        _ => Ok(ThreadResponse::Bool(true)),
    }
}

pub(crate) fn get_status(relays: &Mutex<HashMap<String, KasaPlug>>) -> Result<Value, Error> {
    let mut result: Value = json!({});
    let mut relay_statuses: Vec<Value> = Vec::new();
    let mut rooms: HashSet<String> = HashSet::new();

    for (relay_name, relay) in relays.lock().expect("Error getting global RELAYS").iter() {
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
    let thread = thread::spawn(move || {
        let temp_config = load_config().unwrap();

        let relays: Mutex<HashMap<String, KasaPlug>> = Mutex::new(temp_config.relays);
        let presets: Mutex<HashMap<String, Preset>> = Mutex::new(temp_config.presets);

        println!("Relays:");
        for (relay_name, _) in relays.lock().unwrap().iter() {
            println!("\t{relay_name}");
        }

        println!("Presets:");
        for (preset_name, _) in presets.lock().unwrap().iter() {
            println!("\t{preset_name}");
        }

        for received in receiver {
            let response =
                handle_command(received, &relays, &presets).expect("TODO: panic message");
            sender
                .send(ThreadPackage::Response(response))
                .expect("TODO: panic message");
        }
    });
    thread
}
