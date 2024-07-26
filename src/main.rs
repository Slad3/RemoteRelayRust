#![allow(unused)]

mod models;
mod routes;
mod utils;

use std::io::{Error, ErrorKind};
use std::ops::Deref;
use std::string::ToString;
use std::sync::Arc;
use std::sync::Mutex;
use std::vec;

use serde_json::{Value, json};

use rocket::http::uri::fmt::UriQueryArgument::Raw;
use rocket::request::FromRequest;
use rocket::response::content;
use rocket::response::content::{RawHtml, RawJson, RawText};
use rocket::yansi::Paint;
use rocket::{futures, tokio, Build, Request, Rocket, State};
use serde_json::map::Values;
use models::relays::KasaPlug;
use models::presets::Preset;
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
    let mut result = json!({});
    unsafe {
        for relay in RELAYS.lock().expect("Error getting global RELAYS").iter() {
            let id = relay.ip.to_string();
            result[id] = relay.to_json();
        }
    }
    RawJson(result.to_string())
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
        Ok(initial_setup) => RawJson(json!( {"Refreshed": true}).to_string()),
        Err(error) => RawJson(json!( {"Refreshed": false}).to_string())
    }
}


#[get("/getPresets")]
fn get_presets_route() -> RawJson<String> {
    let mut result: Vec<Value> = Vec::new();
    unsafe {
        for preset in PRESETS.lock().expect("Error getting global PRESETS").iter() {
            result.push(preset.to_json());
        }
    }
    RawJson(serde_json::to_string(&result).expect("Penis"))
}


#[get("/setPreset/<preset_name>")]
fn set_preset_route(preset_name: String) -> RawJson<String> {
    let mut found = false;
    unsafe {
        for pres in PRESETS.lock().expect("Error getting global PRESETS").iter() {
            println!("{} {} {}", pres.name.to_lowercase() == preset_name.to_lowercase(), pres.name.to_lowercase(), preset_name.to_lowercase());
            if (pres.name.to_lowercase() == preset_name.to_lowercase()) {
                set_preset(pres);
                found = true;
                break;
            }
        }
    }

    match found {
        true => RawJson(json!( {"PresetSet": true}).to_string()),
        false => RawJson(json!( {"Error": "Could not find preset name in presets"}).to_string())
    }
}

#[get("/setRelay/<relay_name>/<value>")]
fn set_relay_route(relay_name: String, value: bool) -> RawJson<String> {
    let mut found = false;
    let result = set_relay(&relay_name, &value);

    match result {
        Ok(result) => RawJson(json!( {"RelaySet": true}).to_string()),
        Err(error) => RawJson(json!( {"Error": "Could not find preset name in presets"}).to_string())
    }
}


fn set_preset(preset: &Preset) {
    println!("{:?}", preset.relays);
    for (rel, value) in preset.relays.iter() {
        set_relay(rel, value);
    }
}

fn set_relay(relay_name: &String, value: &bool) -> Result<bool, Error> {
    let mut found = false;
    unsafe {
        let mut relays = RELAYS.lock().expect("Error getting global RELAYS");
        for relay in relays.iter_mut() {
            if (relay.name.to_lowercase() == relay_name.to_lowercase()) {
                found = true;
                match value {
                    true => {let _ = relay.turn_on().expect("Can't Connect to Plug");},
                    false => {let _ = relay.turn_off().expect("Can't Connect to Plug");}
                }
            }
        }
    }

    match found {
        true => Ok(true),
        false => Err(Error::new(ErrorKind::Other, "Can't find Relay".to_string())),
    }
}

fn setup() -> Result<bool, Error>{
    let config = load_config()?;

    unsafe {
        RELAYS = Mutex::new(config.relays);
        PRESETS = Mutex::new(config.presets);
    }

    Ok(true)
}


#[launch]
fn rocket() -> _ {

    let initial_setup = setup();
    match initial_setup {
        Ok(initial_setup) => {println!("Initial Setup Successful")},
        Err(error) => panic!("{}", format!("Initial setup failed {error}"))
    }

    rocket::build().mount("/",
                          routes![
                            index_state,
                            status_route,
                            switch_route,
                            get_presets_route,
                            set_preset_route])
}
