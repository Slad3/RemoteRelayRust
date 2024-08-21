use crate::models::config_models::Config;
use crate::utils::local_config_utils::load_local_config;

use std::io::Error;

#[derive(Clone, Copy)]
pub(crate) enum ConfigLocation {
    MONGODB,
    LOCAL,
}

pub fn load_config(config_location: ConfigLocation) -> Result<Config, Error> {
    match config_location {
        ConfigLocation::LOCAL | _ => load_local_config(),
        // ConfigLocation::MONGODB => {}
    }
}
