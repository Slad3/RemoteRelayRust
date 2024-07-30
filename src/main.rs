mod models;
mod routes;
mod utils;

use std::collections::HashSet;
use std::io::{Error, ErrorKind};
use std::string::ToString;
use std::sync::Mutex;
use std::vec;

use serde_json::{json, Value};

use models::presets::Preset;
use models::relays::KasaPlug;

use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;
use rocket::response::content::RawJson;
use rocket::{Request, Response};

use utils::local_config_utils::load_config;

#[macro_use]
extern crate rocket;

static mut RELAYS: Mutex<Vec<KasaPlug>> = Mutex::new(Vec::new());
static mut PRESETS: Mutex<Vec<Preset>> = Mutex::new(Vec::new());

#[get("/")]
fn index_state() -> RawJson<String> {
    RawJson(json!( {"HealthCheck": true}).to_string())
}

#[get("/status")]
fn status_route() -> RawJson<String> {
    let status: Result<Value, Error> = get_status();

    match status {
        Ok(result) => RawJson(result.to_string()),
        Err(error) => {
            RawJson(json!( {"Error": format!("Could not get status {}", error)}).to_string())
        }
    }
}

#[get("/switch")]
pub fn switch_route() -> RawJson<String> {
    unsafe {
        let mut relays = RELAYS.lock().expect("Error getting global RELAYS");

        for relay in relays.iter_mut() {
            let _ = relay.switch();
        }
    }

    RawJson(json!( {"Switched": true}).to_string())
}

#[get("/refresh")]
fn refresh_route() -> RawJson<String> {
    let initial_setup = setup();
    match initial_setup {
        Ok(..) => RawJson(json!( {"Refreshed": true}).to_string()),
        Err(..) => RawJson(json!( {"Refreshed": false}).to_string()),
    }
}

#[get("/preset/getPresets")]
fn get_presets_route() -> RawJson<String> {
    let mut result: Vec<Value> = Vec::new();
    unsafe {
        for preset in PRESETS.lock().expect("Error getting global PRESETS").iter() {
            result.push(preset.to_json());
        }
    }
    RawJson(serde_json::to_string(&result).expect("Penis"))
}

#[get("/preset/getPresetNames")]
fn get_preset_names_route() -> RawJson<String> {
    let mut result: Vec<String> = Vec::new();
    unsafe {
        for preset in PRESETS.lock().expect("Error getting global PRESETS").iter() {
            result.push(preset.name.to_string());
        }
    }

    RawJson(serde_json::to_string(&result).expect("Penis"))
}

#[get("/preset/setPreset/<preset_name>")]
fn set_preset_route(preset_name: String) -> RawJson<String> {
    let mut found = false;
    unsafe {
        for pres in PRESETS.lock().expect("Error getting global PRESETS").iter() {
            if pres.name.to_lowercase() == preset_name.to_lowercase() {
                set_preset(pres);
                found = true;
                break;
            }
        }
    }

    match found {
        true => RawJson(json!( {"PresetSet": true}).to_string()),
        false => RawJson(json!( {"Error": "Could not find preset name in presets"}).to_string()),
    }
}

#[get("/setRelay/<relay_name>/<value>")]
fn set_relay_route(relay_name: String, value: bool) -> RawJson<String> {
    let result = set_relay(&relay_name, &value);

    match result {
        Ok(result) => RawJson(json!( {"RelaySet": result}).to_string()),
        Err(error) => RawJson(
            json!( {"Error": format!("Could not find preset name in presets {}", error)})
                .to_string(),
        ),
    }
}

fn set_preset(preset: &Preset) {
    unsafe {
        let mut relays = RELAYS.lock().expect("Error getting global RELAYS");
        for relay in relays.iter_mut() {
            let rel = preset.relays.get_key_value(&relay.name);
            match rel {
                Some(temp) => {
                    let (_, &value) = temp;
                    if value {
                        relay.turn_on().expect("Can't Connect to Plug");
                    } else {
                        relay.turn_on().expect("Can't Connect to Plug");
                    }
                }
                None => {
                    println!("Not found relay {}", relay.name);
                    let _ = relay.turn_off().expect("Can't Connect to Plug");
                }
            }
        }
    }
}

fn set_relay(relay_name: &String, value: &bool) -> Result<bool, Error> {
    let mut found = false;
    unsafe {
        let mut relays = RELAYS.lock().expect("Error getting global RELAYS");
        for relay in relays.iter_mut() {
            if relay.name.to_lowercase() == relay_name.to_lowercase() {
                found = true;
                match value {
                    true => {
                        let _ = relay.turn_on().expect("Can't Connect to Plug");
                    }
                    false => {
                        let _ = relay.turn_off().expect("Can't Connect to Plug");
                    }
                }
            }
        }
    }

    match found {
        true => Ok(true),
        false => Err(Error::new(ErrorKind::Other, "Can't find Relay".to_string())),
    }
}

fn get_status() -> Result<Value, Error> {
    let mut result: Value = json!({});
    let mut relays: Vec<Value> = Vec::new();
    let mut rooms: HashSet<String> = HashSet::new();
    unsafe {
        for relay in RELAYS.lock().expect("Error getting global RELAYS").iter() {
            relays.push(relay.to_json());
            rooms.insert(relay.room.clone());
        }
    }

    result["relays"] = Value::Array(relays);
    result["rooms"] = Value::Array(rooms.into_iter().map(Value::String).collect());

    Ok(result)
}

fn setup() -> Result<bool, Error> {
    let config = load_config()?;

    unsafe {
        RELAYS = Mutex::new(config.relays);
        PRESETS = Mutex::new(config.presets);
    }

    Ok(true)
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

#[launch]
fn rocket() -> _ {
    let initial_setup = setup();
    match initial_setup {
        Ok(..) => {
            println!("Initial Setup Successful")
        }
        Err(error) => panic!("{}", format!("Initial setup failed {error}")),
    }

    rocket::build().attach(Cors).mount(
        "/",
        routes![
            index_state,
            status_route,
            switch_route,
            refresh_route,
            get_presets_route,
            set_preset_route,
            get_preset_names_route,
            set_relay_route
        ],
    )
}
