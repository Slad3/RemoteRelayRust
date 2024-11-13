use crate::models::presets::Preset;
use crate::models::relays::RelayType;
use rocket::serde::{Deserialize, Serialize};
use std::collections::HashMap;

fn empty_list() -> Vec<String> {
    Vec::new()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub(crate) relays: HashMap<String, RelayType>,
    pub(crate) presets: HashMap<String, Preset>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ConfigRelayType {
    KasaPlug,
    KasaMultiPlug,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ConfigRelay {
    #[serde(rename = "type")]
    pub(crate) relay_type: ConfigRelayType,
    #[serde(default)]
    pub(crate) name: String,
    #[serde(default = "empty_list")]
    pub(crate) names: Vec<String>,
    pub(crate) ip: String,
    pub(crate) room: String,
    #[serde(default = "empty_list")]
    pub(crate) tags: Vec<String>,
}
