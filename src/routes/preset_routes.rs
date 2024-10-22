use crate::models::api_response::ApiResponse;
use crate::models::channels_models::Channels;
use crate::models::data_thread_models::{
    DataThreadCommand::Preset, DataThreadResponse, PresetCommand,
};
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use serde_json::json;
#[get("/preset/set/<preset_name>")]
pub(crate) async fn set_preset_route(preset_name: &str, channels: &State<Channels>) -> ApiResponse {
    if channels
        .route_to_data_sender
        .send(Preset(PresetCommand::Set(preset_name.parse().unwrap())))
        .is_err()
    {
        return ApiResponse {
            value: Json(json!({"Error": "Could not set preset"})),
            status: Status::ExpectationFailed,
        };
    }

    match channels
        .data_to_route_receiver
        .lock()
        .expect("Got data from channel")
        .recv()
    {
        Ok(DataThreadResponse::Value(result)) => ApiResponse {
            value: Json(result),
            status: Status::Ok,
        },
        Err(_) | _ => ApiResponse {
            value: Json(
                json!({"Error": format!("Could not find preset to set: {}", &preset_name)}),
            ),
            status: Status::NotFound,
        },
    }
}

#[get("/preset/getPresetNames")]
pub(crate) async fn get_preset_names_route(channels: &State<Channels>) -> ApiResponse {
    let error_message = ApiResponse {
        value: Json(json!({"Error": "Could not get preset names"})),
        status: Status::ExpectationFailed,
    };

    if channels
        .route_to_data_sender
        .send(Preset(PresetCommand::Names))
        .is_err()
    {
        return error_message;
    }

    match channels
        .data_to_route_receiver
        .lock()
        .expect("Got data from channel")
        .recv()
    {
        Ok(DataThreadResponse::Value(final_response)) => ApiResponse {
            value: Json(final_response),
            status: Status::Ok,
        },
        _ => error_message,
    }
}
