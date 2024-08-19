use crate::models::api_response::ApiResponse;
use crate::utils::thread_handling::ThreadCommand::Preset;
use crate::utils::thread_handling::{PresetCommand, ThreadPackage, ThreadResponse};
use crate::Channels;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use serde_json::json;
#[get("/preset/set/<preset_name>")]
pub(crate) fn set_preset_route(preset_name: String, channels: &State<Channels>) -> ApiResponse {
    if channels
        .route_to_data_sender
        .send(ThreadPackage::ThreadCommand(Preset(PresetCommand::Set(
            preset_name.clone(),
        ))))
        .is_err()
    {
        return ApiResponse {
            value: Json(json!({"Error": "Could not set preset"})),
            status: Status::ExpectationFailed,
        };
    }

    let res = channels.data_to_route_receiver.lock().unwrap().recv();
    match res {
        Ok(ThreadPackage::ThreadResponse(ThreadResponse::Value(final_response))) => ApiResponse {
            value: Json(final_response),
            status: Status::Ok,
        },
        _ => ApiResponse {
            value: Json(
                json!({"Error": format!("Could not find preset to set: {}", &preset_name)}),
            ),
            status: Status::NotFound,
        },
    }
}

#[get("/preset/getPresetNames")]
pub(crate) fn get_preset_names_route(channels: &State<Channels>) -> ApiResponse {
    let error_message = ApiResponse {
        value: Json(json!({"Error": "Could not get preset names"})),
        status: Status::ExpectationFailed,
    };

    if channels
        .route_to_data_sender
        .send(ThreadPackage::ThreadCommand(Preset(PresetCommand::Names)))
        .is_err()
    {
        return error_message;
    }

    let res = channels.data_to_route_receiver.lock().unwrap().recv();
    match res {
        Ok(ThreadPackage::ThreadResponse(ThreadResponse::Value(final_response))) => ApiResponse {
            value: Json(final_response),
            status: Status::Ok,
        },
        _ => error_message,
    }
}
