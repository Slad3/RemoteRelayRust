use crate::models::config_models::Config;
use crate::utils::local_config_utils::load_local_config;
use crate::utils::mongodb_utils::load_mongo_config;
use std::io::{Error, ErrorKind};
use std::thread;
use std::thread::JoinHandle;
use tokio::runtime::Runtime;

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

pub fn load_config(config_location: ConfigLocation) -> JoinHandle<Result<Config, Error>> {
    thread::spawn(move || {
        let rt = Runtime::new().expect("Could not create runtime");

        match config_location {
            ConfigLocation::MONGODB => {
                let mongodb_config = rt.block_on(load_mongo_config());

                match mongodb_config {
                    Ok(config) => Ok(config),
                    Err(error) => Err(Error::new(ErrorKind::Other, error)),
                }
            }

            ConfigLocation::LOCAL => load_local_config(),
        }
    })
}
