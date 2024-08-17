mod models;
mod routes;
mod utils;

use std::collections::HashSet;
use std::io::{Error, ErrorKind};
use std::string::ToString;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, Mutex};
use std::vec;

use serde_json::{json, Value};

use models::relays::KasaPlug;

use crate::routes::preset_routes::{get_preset_names_route, set_preset_route};
use crate::routes::relay_routes::set_relay_command_route;
use crate::utils::thread_handling::ThreadResponse;
use models::presets::{set_preset, Preset};
use models::relays::Relay;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::{Header, Status};
use rocket::request::{FromRequest, Outcome};
use rocket::response::content::RawJson;
use rocket::{Request, Response};
use utils::local_config_utils::load_config;
use utils::thread_handling::{setup_data_thread, ThreadPackage};

#[macro_use]
extern crate rocket;

#[get("/")]
fn index_state() -> RawJson<String> {
    RawJson(json!( {"HealthCheck": true}).to_string())
}

// #[get("/status")]
// fn status_route() -> RawJson<String> {
//     let status: Result<Value, Error> = get_status();
//
//     match status {
//         Ok(result) => RawJson(result.to_string()),
//         Err(error) => {
//             RawJson(json!( {"Error": format!("Could not get status {}", error)}).to_string())
//         }
//     }
// }

// #[get("/switch")]
// pub fn switch_route() -> RawJson<String> {
//     unsafe {
//         let mut relays = RELAYS.lock().expect("Error getting global RELAYS");
//
//         for relay in relays.iter_mut() {
//             let _ = relay.switch();
//         }
//     }
//
//     RawJson(json!( {"Switched": true}).to_string())
// }
//
// #[get("/refresh")]
// fn refresh_route() -> RawJson<String> {
//     let initial_setup = setup();
//     match initial_setup {
//         Ok(..) => RawJson(json!( {"Refreshed": true}).to_string()),
//         Err(..) => RawJson(json!( {"Refreshed": false}).to_string()),
//     }
// }

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
            // status_route,
            // switch_route,
            // refresh_route,
            set_preset_route,
            get_preset_names_route,
            set_relay_command_route,
        ],
    );

    server
}
