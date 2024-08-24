mod models;
mod routes;
mod utils;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, Mutex};
use std::{env, vec};

use crate::models::api_response::ApiResponse;
use crate::routes::preset_routes::{get_preset_names_route, set_preset_route};
use crate::routes::relay_routes::set_relay_command_route;

use crate::models::data_thread_models::{
    DataThreadCommand,
    DataThreadCommand::{Refresh, SystemStatus},
    DataThreadResponse,
};
use crate::utils::data_thread_handling::setup_data_thread;

use crate::utils::load_config::ConfigLocation;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::{Header, Status};
use rocket::serde::json::Json;
use rocket::{Request, Response, State};
use serde_json::json;

#[macro_use]
extern crate rocket;

#[get("/")]
fn index_state() -> ApiResponse {
    ApiResponse {
        value: Json(json!( {"HealthCheck": true})),
        status: Status::Ok,
    }
}

#[get("/status")]
fn status_route(channels: &State<Channels>) -> ApiResponse {
    let error_message = ApiResponse {
        value: Json(json!({"Error": "Could not get preset names"})),
        status: Status::new(500),
    };

    if channels.route_to_data_sender.send(SystemStatus).is_err() {
        return error_message;
    }

    let res = channels.data_to_route_receiver.lock().unwrap().recv();
    match res {
        Ok(DataThreadResponse::Value(final_response)) => ApiResponse {
            value: Json(final_response),
            status: Status::Ok,
        },
        _ => error_message,
    }
}

#[get("/refresh")]
fn refresh_route(channels: &State<Channels>) -> ApiResponse {
    let error_message = ApiResponse {
        value: Json(json!({"Error": "Could not refresh config"})),
        status: Status::new(500),
    };

    if channels.route_to_data_sender.send(Refresh).is_err() {
        return error_message;
    }

    let res = channels.data_to_route_receiver.lock().unwrap().recv();
    match res {
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

pub struct Cors;

#[rocket::async_trait]
impl Fairing for Cors {
    fn info(&self) -> Info {
        Info {
            name: "Cross-Origin-Resource-Sharing Fairing",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, PATCH, PUT, DELETE, HEAD, OPTIONS, GET",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

#[derive(Debug)]
struct Channels {
    route_to_data_sender: Sender<DataThreadCommand>,
    data_to_route_receiver: Arc<Mutex<Receiver<DataThreadResponse>>>,
}

#[launch]
async fn rocket() -> _ {
    let args: Vec<String> = env::args().collect();

    let mut config_location = ConfigLocation::MONGODB;
    if args.contains(&"LOCAL_CONFIG".to_string()) {
        config_location = ConfigLocation::LOCAL
    }

    println!("Loading config from: {config_location}");

    let (route_to_data_sender, route_to_data_receiver) = mpsc::channel::<DataThreadCommand>();
    let (data_to_route_sender, data_to_route_receiver) = mpsc::channel::<DataThreadResponse>();

    let channels = Channels {
        route_to_data_sender,
        data_to_route_receiver: Arc::new(Mutex::new(data_to_route_receiver)),
    };

    let data_thread = setup_data_thread(
        data_to_route_sender,
        route_to_data_receiver,
        config_location,
    );

    let _ = data_thread.thread();

    let server = rocket::build().attach(Cors).manage(channels).mount(
        "/",
        routes![
            index_state,
            status_route,
            refresh_route,
            set_preset_route,
            get_preset_names_route,
            set_relay_command_route,
        ],
    );

    server
}
