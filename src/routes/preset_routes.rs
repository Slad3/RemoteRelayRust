use crate::utils::thread_handling::ThreadCommand::Preset;
use crate::utils::thread_handling::{PresetCommand, ThreadPackage, ThreadResponse};
use crate::Channels;
use rocket::response::content::RawJson;
use rocket::serde::json::Json;
use rocket::State;
use serde_json::{json, Value};

#[get("/preset/set/<preset_name>")]
pub(crate) fn set_preset_route(preset_name: String, channels: &State<Channels>) -> Json<Value> {
    let result = channels
        .route_to_data_sender
        .send(ThreadPackage::ThreadCommand(Preset(PresetCommand::Set(
            preset_name,
        ))));

    match result {
        Ok(result) => Json(json!( {"PresetSet": result})),
        Err(error) => {
            Json(json!( {"Error": format!("Could not find preset name in presets {}", error)}))
        }
    }
}

#[get("/preset/getPresetNames")]
pub(crate) fn get_preset_names_route(channels: &State<Channels>) -> Json<Value> {
    let error_message = Json(json!({"Error": "Could not get preset names"}));

    if channels
        .route_to_data_sender
        .send(ThreadPackage::ThreadCommand(Preset(PresetCommand::Names)))
        .is_err()
    {
        return error_message;
    }

    let res = channels.data_to_route_receiver.lock().unwrap().recv();
    match res {
        Ok(ThreadPackage::Response(ThreadResponse::Value(final_response))) => Json(final_response),
        _ => error_message,
    }
}
