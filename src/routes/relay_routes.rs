use crate::models::data_thread_models::{DataThreadCommand::Relay, RelayCommand};
use crate::utils::data_thread_handling::{handle_command_input, unwrap_response};
use crate::Channels;

use crate::models::api_response::ApiResponse;
use serde_json::json;

use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;

#[get("/relay/<relay_name>/<command_input>")]
pub(crate) fn set_relay_command_route(
    relay_name: &str,
    command_input: &str,
    channels: &State<Channels>,
) -> ApiResponse {
    let command_processed = match handle_command_input(command_input) {
        Some(command) => command,
        None => {
            return ApiResponse {
                value: Json(json!({"Error": "Could not process command"})),
                status: Status::NotAcceptable,
            }
        }
    };

    if let Err(_) = channels.route_to_data_sender.send(Relay(RelayCommand {
        name: relay_name.parse().unwrap(),
        command: command_processed,
    })) {
        return ApiResponse {
            value: Json(json!({"Error": "Channel closed"})),
            status: Status::new(500),
        };
    }

    match channels.data_to_route_receiver.lock().unwrap().recv() {
        Ok(response) => ApiResponse {
            value: unwrap_response(response),
            status: Status::Ok,
        },
        Err(error) => ApiResponse {
            value: Json(json!({"Error": format!("Could not find relay name in relays {}", error)})),
            status: Status::NotFound,
        },
    }
}
