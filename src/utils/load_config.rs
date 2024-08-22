use crate::models::config_models::Config;
use crate::utils::local_config_utils::load_local_config;
use crate::utils::mongodb_utils::load_mongo_config;
use std::io::{Error, ErrorKind};

use futures::executor::block_on;

#[derive(Debug, Clone, Copy)]
pub(crate) enum ConfigLocation {
    MONGODB,
    LOCAL,
}

impl std::fmt::Display for ConfigLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let location = match *self {
            ConfigLocation::MONGODB => "MongoDB",
            ConfigLocation::LOCAL => "Local",
        };
        write!(f, "{}", location)
    }
}

pub fn load_config(config_location: ConfigLocation) -> Result<Config, Error> {
    match config_location {
        ConfigLocation::MONGODB => {
            let config = block_on(load_mongo_config());
            match config {
                Ok(config) => Ok(config),
                Err(error) => Err(Error::new(ErrorKind::Other, error)),
            }
        }
        ConfigLocation::LOCAL => load_local_config(),
    }
}
