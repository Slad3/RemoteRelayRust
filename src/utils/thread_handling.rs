use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::models::presets::{set_preset, Preset};
use crate::models::relays::KasaPlug;
use crate::utils::local_config_utils::load_config;
use std::io::Error;
use std::sync::mpsc::{Receiver, SendError};
use std::sync::{Mutex};
use std::thread;
use std::thread::JoinHandle;

#[derive(Debug)]
// pub(crate) enum ThreadResponse {
pub(crate) enum ThreadPackage {
    ThreadCommand(ThreadCommand),
    Response,
}

#[derive(Debug)]
enum ThreadCommand {
    Status,
    Switch,
    AllOff,
    Relay(RelayCommand),
    Preset(PresetCommand),
}

#[derive(Debug)]
struct RelayCommand {
    name: String,
    command: RelayCommands,
}

#[derive(Debug)]
struct PresetCommand {
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
enum RelayCommands {
    #[serde(rename = "true")]
    TRUE,
    FALSE,
    SWITCH,
    STATUS,
}

fn handle_send(result: Result<(), SendError<ThreadPackage>>) {
    match result {
        Err(error) => {
            println!("{}", error.to_string())
        }
        _ => {}
    }
}

fn handle_received(thread_name: &str, received: ThreadPackage) {
    match received {
        ThreadPackage::ThreadCommand(rec) => {
            println!("{thread_name}:\t{:?}", rec);
        }
        _ => {}
    }
}

fn handle_command_string(input: &str) -> Option<RelayCommands> {
    match input {
        "TRUE" => Option::from(RelayCommands::TRUE),
        "FALSE" => Option::from(RelayCommands::FALSE),
        "SWITCH" => Option::from(RelayCommands::SWITCH),
        "STATUS" => Option::from(RelayCommands::STATUS),
        _ => Option::None,
    }
}
fn handle_command(
    received: ThreadPackage,
    relays: &Mutex<HashMap<String, KasaPlug>>,
    presets: &Mutex<HashMap<String, Preset>>,
) -> Result<bool, Error> {
    match received {
        ThreadPackage::ThreadCommand(command) => match command {
            ThreadCommand::Relay(relay_command) => {
                println!("Relay Command:\t{:?}", relay_command);

                if let Some(mut relay) = relays.lock().unwrap().get_mut(&relay_command.name) {
                    match relay_command.command {
                        RelayCommands::SWITCH => relay.switch(),
                        RelayCommands::TRUE => relay.turn_on(),
                        RelayCommands::FALSE => relay.turn_off(),
                        RelayCommands::STATUS => relay.get_status(),
                    }
                } else {
                    Ok(false)
                }
            }
            ThreadCommand::Preset(preset_command) => {
                println!("Preset_command:\t{:?}", preset_command);
                if let Some(mut preset) = presets.lock().unwrap().get_mut(&preset_command.name) {
                    let _ = set_preset(&preset.name, presets, relays);
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            ThreadCommand::Status => Ok(true),
            ThreadCommand::Switch => Ok(true),
            ThreadCommand::AllOff => Ok(true)
        },
        _ => Ok(true),
    }
}

pub(crate) fn setup_data_thread(receiver: Receiver<ThreadPackage>) -> JoinHandle<()> {
    let thread = thread::spawn(move || {
        let temp_config = load_config().unwrap();

        let relays: Mutex<HashMap<String, KasaPlug>> = Mutex::new(temp_config.relays);
        let presets: Mutex<HashMap<String, Preset>> = Mutex::new(temp_config.presets);

        for (relay_name, _) in relays.lock().unwrap().iter() {
            println!("{relay_name}");
        }

        for received in receiver {
            handle_command(received, &relays, &presets).expect("TODO: panic message");
        }
    });
    thread
}
