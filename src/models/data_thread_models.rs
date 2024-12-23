use rocket::serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum DataThreadResponse {
    Value(Value),
    Bool(bool),
    Error(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) enum DataThreadCommand {
    SystemStatus,
    AutoRefresh,
    Refresh,
    Relay(RelayCommand),
    Tag(TagCommand),
    Preset(PresetCommand),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct TagCommand {
    pub(crate) tag: String,
    pub(crate) command: RelayCommands,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct RelayCommand {
    pub(crate) name: String,
    pub(crate) command: RelayCommands,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) enum PresetCommand {
    Set(String),
    Names,
    // CurrentPreset,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) enum RelayCommands {
    #[serde(rename = "true")]
    TRUE,
    #[serde(rename = "false")]
    FALSE,
    SWITCH,
    STATUS
}
