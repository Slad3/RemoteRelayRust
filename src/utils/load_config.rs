use crate::models::config_models::Config;
use crate::utils::local_config_utils::load_local_config;
use crate::utils::mongodb_utils::load_mongo_config;
use std::io::{Error, ErrorKind};

#[derive(Debug, Clone, Copy)]
pub(crate) enum ConfigLocation {
    MONGODB,
    LOCAL,
}

pub fn load_config(config_location: ConfigLocation) -> Result<Config, Error> {
    match config_location {
        ConfigLocation::MONGODB => {
            let config = tokio::spawn(async { load_mongo_config().await });

            // Handle the result of the spawned task
            let config = tokio::runtime::Handle::current().block_on(config)?;

            match config {
                Ok(config) => Ok(config),
                Err(error) => Err(Error::new(ErrorKind::Other, error)),
            }
        }
        ConfigLocation::LOCAL => load_local_config(),
    }
}
