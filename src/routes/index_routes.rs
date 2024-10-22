use crate::models::api_response::ApiResponse;
use crate::models::channels_models::Channels;
use crate::models::data_thread_models::DataThreadCommand::{Refresh, SystemStatus};
use crate::models::data_thread_models::DataThreadResponse;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use serde_json::json;

#[get("/")]
pub async fn index_route() -> ApiResponse {
    ApiResponse {
        value: Json(json!( {"HealthCheck": true})),
        status: Status::Ok,
    }
}

#[get("/status")]
pub async fn status_route(channels: &State<Channels>) -> ApiResponse {
    let error_message = ApiResponse {
        value: Json(json!({"Error": "Could not get preset names"})),
        status: Status::new(500),
    };

    if channels.route_to_data_sender.send(SystemStatus).is_err() {
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

#[get("/refresh")]
pub async fn refresh_route(channels: &State<Channels>) -> ApiResponse {
    let error_message = ApiResponse {
        value: Json(json!({"Error": "Could not refresh config"})),
        status: Status::new(500),
    };

    if channels.route_to_data_sender.send(Refresh).is_err() {
        return error_message;
    }

    match channels
        .data_to_route_receiver
        .lock()
        .expect("Got data from channel")
        .recv()
    {
        Ok(DataThreadResponse::Bool(final_response)) => ApiResponse {
            value: Json(json!({"refresh" : final_response})),
            status: Status::Ok,
        },
        Ok(DataThreadResponse::Error(final_response)) => ApiResponse {
            value: Json(json!({"Error": format!("{:?}", final_response)})),
            status: Status::new(500),
        },
        _ => error_message,
    }
}
