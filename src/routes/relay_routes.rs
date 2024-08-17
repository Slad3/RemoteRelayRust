use crate::utils::thread_handling::{
    handle_command_input, unwrap_response, RelayCommand, ThreadCommand::Relay, ThreadPackage,
};
use crate::Channels;
use rocket::response::content::RawJson;
use rocket::State;
use serde_json::json;

#[get("/relay/<relay_name>/<command_input>")]
pub(crate) fn set_relay_command_route(
    relay_name: String,
    command_input: String,
    channels: &State<Channels>,
) -> RawJson<String> {
    let command_processed = match handle_command_input(command_input) {
        Some(command) => command,
        None => return RawJson(json!({"Error": "Could not process command"}).to_string()),
    };

    if let Err(_) = channels
        .route_to_data_sender
        .send(ThreadPackage::ThreadCommand(Relay(RelayCommand {
            name: relay_name.clone(),
            command: command_processed,
        })))
    {
        return RawJson(json!({"Error": "Could not fetch from data"}).to_string());
    }

    match channels.data_to_route_receiver.lock().unwrap().recv() {
        Ok(response) => unwrap_response(response),
        Err(error) => RawJson(
            json!({"Error": format!("Could not find relay name in relays {}", error)}).to_string(),
        ),
    }
}
