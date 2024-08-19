use rocket::serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum ThreadPackage {
    ThreadCommand(ThreadCommand),
    ThreadResponse(ThreadResponse),
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum ThreadResponse {
    Value(Value),
    Bool(bool),
    Error(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum ThreadCommand {
    SystemStatus,
    Refresh,
    Relay(RelayCommand),
    Preset(PresetCommand),
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct RelayCommand {
    pub(crate) name: String,
    pub(crate) command: RelayCommands,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum PresetCommand {
    Set(String),
    Names,
    // CurrentPreset,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum RelayCommands {
    #[serde(rename = "true")]
    TRUE,
    FALSE,
    SWITCH,
    STATUS,
}
