use crate::utils::thread_handling::ThreadCommand::Preset;
use crate::utils::thread_handling::{PresetCommand, ThreadPackage, ThreadResponse};
use crate::Channels;
use rocket::response::content::RawJson;
use rocket::State;
use serde_json::json;

#[get("/preset/set/<preset_name>")]
pub(crate) fn set_preset_route(preset_name: String, channels: &State<Channels>) -> RawJson<String> {
    let result = channels
        .route_to_data_sender
        .send(ThreadPackage::ThreadCommand(Preset(PresetCommand::Set(
            preset_name,
        ))));

    match result {
        Ok(result) => RawJson(json!( {"PresetSet": result}).to_string()),
        Err(error) => RawJson(
            json!( {"Error": format!("Could not find preset name in presets {}", error)})
                .to_string(),
        ),
    }
}

#[get("/preset/getPresetNames")]
pub(crate) fn get_preset_names_route(channels: &State<Channels>) -> RawJson<String> {
    let error_message = RawJson(json!({"Error": "Could not get preset names"}).to_string());

    if channels
        .route_to_data_sender
        .send(ThreadPackage::ThreadCommand(Preset(PresetCommand::Names)))
        .is_err()
    {
        return error_message;
    }

    let res = channels.data_to_route_receiver.lock().unwrap().recv();
    match res {
        Ok(ThreadPackage::Response(ThreadResponse::Value(final_response))) => {
            println!("{:?}", &final_response);
            RawJson(final_response.to_string())
        }
        _ => error_message,
    }
}
