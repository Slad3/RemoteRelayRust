mod models;
mod routes;
mod utils;

use std::sync::{mpsc, Arc, Mutex};
use std::vec;

use crate::routes::index_routes::{index_route, refresh_route, status_route};
use crate::routes::preset_routes::{get_preset_names_route, set_preset_route};
use crate::routes::relay_routes::{set_relay_command_route, set_relays_by_tag_command_route};

use crate::models::channels_models::Channels;
use crate::models::data_thread_models::{DataThreadCommand, DataThreadResponse};
use crate::models::rocket_cors::Cors;
use crate::utils::data_thread_handling::setup_data_thread;
use crate::utils::load_config::ConfigLocation;
use clap::Parser;

#[macro_use]
extern crate rocket;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    config: Option<String>,
}

fn get_config_location(args: Args) -> ConfigLocation {
    match args.config.unwrap_or("local".to_string()).as_str() {
        "local" => ConfigLocation::LOCAL,
        "mongodb" | "mongo" => ConfigLocation::MONGODB,
        _ => {
            eprintln!("No config= argument found, defaulting to local");
            ConfigLocation::LOCAL
        }
    }
}

#[launch]
async fn rocket() -> _ {
    let args: Args = Args::parse();

    let config_location = get_config_location(args);

    println!("Loading config from: {config_location}");

    let (route_to_data_sender, route_to_data_receiver) = mpsc::channel::<DataThreadCommand>();
    let (data_to_route_sender, data_to_route_receiver) = mpsc::channel::<DataThreadResponse>();

    // let rtds = route_to_data_sender.clone();
    let channels = Channels {
        route_to_data_sender: route_to_data_sender.clone().clone(),
        data_to_route_receiver: Arc::new(Mutex::new(data_to_route_receiver)),
    };

    let data_thread = setup_data_thread(
        data_to_route_sender,
        route_to_data_receiver,
        route_to_data_sender.clone(),
        config_location,
    );

    let _ = data_thread.thread();

    let server = rocket::build().attach(Cors).manage(channels).mount(
        "/",
        routes![
            index_route,
            status_route,
            refresh_route,
            set_preset_route,
            get_preset_names_route,
            set_relay_command_route,
            set_relays_by_tag_command_route
        ],
    );

    server
}
