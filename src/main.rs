#![allow(unused)]
mod relays;
mod relay_routes;
mod config_helpers;

use std::ops::Deref;
use std::string::ToString;
use std::sync::Mutex;
use std::vec;



use rocket::response::content;
use rocket::response::content::{RawHtml, RawJson, RawText};
use rocket::{Build, Request, Rocket, State};
use rocket::request::FromRequest;
use rocket::http::uri::fmt::UriQueryArgument::Raw;
use relay_routes::external_route;

use relays::{KasaPlug};

use config_helpers::load_config;

#[macro_use] extern crate rocket;


static mut RELAYS: Mutex<Vec<KasaPlug>> = Mutex::new(Vec::new());

#[get("/switch")]
fn index_state() -> RawJson<String> {

    unsafe {
        for relay in RELAYS.lock().unwrap().iter_mut() {
            let _ = relay.switch();
        }
    }

    unsafe {
        RawJson(RELAYS.lock().unwrap().get(0).unwrap().meta().to_string())
    }
}

#[launch]
fn rocket() -> _ {
    // unsafe {
    //     RELAYS.lock().unwrap().push(KasaPlug::new_static("192.168.0.109", "LampLight", "office"));
    // }

    let config = load_config();

    unsafe { RELAYS = Mutex::new(config); }


    rocket::build()
        .mount("/", routes![index_state, external_route])
}


// fn main(){
//     let config = load_config();
//
//     unsafe { RELAYS = Mutex::new(config); }
//
//     unsafe {
//         for relay in RELAYS.lock().unwrap().iter_mut() {
//             // println!("Here: {}", relay.name);
//             // println!("Here: {}", relay.ip);
//             let _ = relay.switch();
//         }
//     }
//
//     // println!("{:?}", config)
// }
