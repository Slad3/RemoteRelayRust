mod models;
mod routes;
mod utils;
use std::string::ToString;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, Mutex};
use std::vec;

use serde_json::json;

use crate::routes::preset_routes::{get_preset_names_route, set_preset_route};
use crate::routes::relay_routes::set_relay_command_route;
use crate::utils::thread_handling::ThreadCommand::{Preset, Refresh, Status};
use crate::utils::thread_handling::{PresetCommand, ThreadResponse};
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;
use rocket::response::content::RawJson;
use rocket::{Request, Response, State};
use utils::thread_handling::{setup_data_thread, ThreadPackage};

#[macro_use]
extern crate rocket;

#[get("/")]
fn index_state() -> RawJson<String> {
    RawJson(json!( {"HealthCheck": true}).to_string())
}

#[get("/status")]
fn status_route(channels: &State<Channels>) -> RawJson<String> {
    let error_message = RawJson(json!({"Error": "Could not get preset names"}).to_string());

    if channels
        .route_to_data_sender
        .send(ThreadPackage::ThreadCommand(Status))
        .is_err()
    {
        return error_message;
    }

    let res = channels.data_to_route_receiver.lock().unwrap().recv();
    match res {
        Ok(ThreadPackage::Response(ThreadResponse::Value(final_response))) => {
            RawJson(final_response.to_string())
        }
        _ => error_message,
    }
}

#[get("/refresh")]
fn refresh_route(channels: &State<Channels>) -> RawJson<String> {
    let error_message = RawJson(json!({"Error": "Could not refresh config"}).to_string());

    if channels
        .route_to_data_sender
        .send(ThreadPackage::ThreadCommand(Refresh))
        .is_err()
    {
        return error_message;
    }

    let res = channels.data_to_route_receiver.lock().unwrap().recv();
    println!("{:?}", &res);
    match res {
        Ok(ThreadPackage::Response(ThreadResponse::Bool(final_response))) => {
            RawJson(final_response.to_string())
        }
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
    route_to_data_sender: Sender<ThreadPackage>,
    data_to_route_receiver: Arc<Mutex<Receiver<ThreadPackage>>>,
}

#[launch]
async fn rocket() -> _ {
    let (route_to_data_sender, route_to_data_receiver) = mpsc::channel::<ThreadPackage>();
    let (data_to_route_sender, data_to_route_receiver) = mpsc::channel::<ThreadPackage>();
    // let (data_send, data_receive) = mpsc::channel::<ThreadPackage>();

    let channels = Channels {
        route_to_data_sender,
        data_to_route_receiver: Arc::new(Mutex::new(data_to_route_receiver)),
    };

    let data_thread = setup_data_thread(data_to_route_sender, route_to_data_receiver);

    let _ = data_thread.await.thread();

    println!("Here");

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
