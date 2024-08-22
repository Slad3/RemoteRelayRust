use crate::models::presets::Preset;
use crate::models::relays::RelayType;
use rocket::serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub(crate) relays: HashMap<String, RelayType>,
    pub(crate) presets: HashMap<String, Preset>,
}
