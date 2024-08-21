use serde_json::Value;

use rocket::serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use std::io::{Error, ErrorKind};
use std::vec;

use crate::utils::kasa_plug_network_functions;

#[derive(Debug, Serialize, Deserialize)]
pub enum RelayType {
    KasaPlug(KasaPlug),
    KasaMultiPlug(KasaMultiPlug),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ConfigRelayType {
    KasaPlug,
    KasaMultiPlug,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct MongodbRelay {
    #[serde(rename = "type")]
    pub(crate) relay_type: ConfigRelayType,
    pub(crate) name: String,
    pub(crate) ip: String,
    pub(crate) room: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct LocalConfigRelay {
    #[serde(rename = "type")]
    pub(crate) relay_type: ConfigRelayType,
    #[serde(default)]
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) names: Vec<String>,
    pub(crate) ip: String,
    pub(crate) room: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KasaPlug {
    pub(crate) ip: String,
    pub(crate) name: String,
    pub(crate) status: bool,
    pub(crate) room: String,
    pub(crate) tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KasaMultiPlug {
    pub(crate) ip: String,
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) status: bool,
    pub(crate) room: String,
    pub(crate) tags: Vec<String>,
}

impl KasaPlug {
    pub fn new(ip: String, name: String, room: String) -> Self {
        let tags: Vec<String> = Vec::new();
        let plug = KasaPlug {
            ip,
            name,
            status: false,
            tags,
            room,
        };
        plug
    }

    pub(crate) fn connected(&mut self) -> Result<bool, Error> {
        self.get_status()
    }

    pub fn to_json(&self) -> Value {
        json!({
            "type": "Kasa Plug",
            "ip": &self.ip.to_string(),
            "name": &self.name.to_string(),
            "status": &self.status,
            "room": &self.room.to_string(),
            "tags": &self.tags,
        })
    }

    pub fn get_status(&mut self) -> Result<bool, Error> {
        let cmd = json!({"system": {"get_sysinfo": {}}});
        let result = kasa_plug_network_functions::send(self.ip.clone(), cmd.to_string());

        match result {
            Ok(result) => {
                let relay_state = result["system"]["get_sysinfo"]["relay_state"]
                    .as_u64()
                    .unwrap_or(0)
                    == 1;
                self.status = relay_state;
                Ok(relay_state)
            }
            Err(..) => Err(Error::new(
                ErrorKind::ConnectionRefused,
                "Can't Connect To Plug".to_string(),
            )),
        }
    }

    pub fn turn_off(&mut self) -> Result<Value, Error> {
        let cmd = json!({"system": {"set_relay_state": {"state": 0}}});
        let result = kasa_plug_network_functions::send(self.ip.clone(), cmd.to_string());
        match result {
            Ok(..) => {
                self.status = false;
                Ok(self.to_json())
            }
            Err(..) => Err(Error::new(
                ErrorKind::ConnectionRefused,
                "Can't Connect To Plug".to_string(),
            )),
        }
    }

    pub fn turn_on(&mut self) -> Result<Value, Error> {
        let cmd = json!({"system": {"set_relay_state": {"state": 1}}});
        let result = kasa_plug_network_functions::send(self.ip.clone(), cmd.to_string());
        match result {
            Ok(..) => {
                self.status = true;
                Ok(self.to_json())
            }
            Err(..) => Err(Error::new(
                ErrorKind::ConnectionRefused,
                "Can't Connect To Plug".to_string(),
            )),
        }
    }

    pub fn switch(&mut self) -> Result<Value, Error> {
        match self.status {
            true => self.turn_off(),
            false => self.turn_on(),
        }
    }
}

impl std::fmt::Display for KasaPlug {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl KasaMultiPlug {
    pub fn new(ip: String, names: Vec<String>, room: String) -> Vec<KasaMultiPlug> {
        let mut plugs: Vec<KasaMultiPlug> = Vec::new();

        let tags: Vec<String> = Vec::new();

        let command = json!({"system": {"get_sysinfo": {}}});
        let response = kasa_plug_network_functions::send(ip.clone(), command.to_string())
            .expect(format!("Unable to connect to KasaMultiPlug {}", ip.clone()).as_str());
        let children_value = &response["system"]["get_sysinfo"]["children"];

        if let Some(children) = children_value.as_array() {
            for (child, name) in children.iter().zip(names.iter()) {
                let id = child["id"].to_string()[1..child["id"].to_string().len() - 1].to_string();
                let state = child["state"].as_u64().unwrap_or(0) == 1;
                plugs.push(KasaMultiPlug {
                    ip: ip.clone(),
                    id: id.clone(),
                    name: name.clone(),
                    status: state,
                    room: room.clone().parse().unwrap(),
                    tags: tags.clone(),
                })
            }
        }

        plugs
    }

    pub(crate) fn connected(&mut self) -> Result<bool, Error> {
        self.get_status()
    }

    pub fn to_json(&self) -> Value {
        json!({
            "type": "Kasa Plug",
            "ip": &self.ip.to_string(),
            "id": &self.id.to_string(),
            "name": &self.name.to_string(),
            "status": &self.status,
            "room": &self.room.to_string(),
            "tags": &self.tags,
        })
    }

    pub fn get_status(&mut self) -> Result<bool, Error> {
        let cmd = json!({"system": {"get_sysinfo": {}}});
        let result = kasa_plug_network_functions::send(self.ip.clone(), cmd.to_string());

        match result {
            Ok(result) => {
                let relay_state = result["system"]["get_sysinfo"].as_u64().unwrap_or(0) == 1;

                self.status = relay_state;
                Ok(true)
            }
            Err(..) => Err(Error::new(
                ErrorKind::ConnectionRefused,
                "Can't Connect To Plug".to_string(),
            )),
        }
    }

    pub fn turn_off(&mut self) -> Result<Value, Error> {
        let cmd = json!({"context": {"child_ids": [self.id.clone()]}, "system": {"set_relay_state": {"state": 0}}});
        let result = kasa_plug_network_functions::send(self.ip.clone(), cmd.to_string());
        match result {
            Ok(..) => {
                self.status = false;
                Ok(self.to_json())
            }
            Err(..) => Err(Error::new(
                ErrorKind::ConnectionRefused,
                "Can't Connect To Plug".to_string(),
            )),
        }
    }

    pub fn turn_on(&mut self) -> Result<Value, Error> {
        let cmd = json!({"context": {"child_ids": [self.id.clone()]}, "system": {"set_relay_state": {"state": 1}}});
        let result = kasa_plug_network_functions::send(self.ip.clone(), cmd.to_string());
        match result {
            Ok(..) => {
                self.status = true;
                Ok(self.to_json())
            }
            Err(..) => Err(Error::new(
                ErrorKind::ConnectionRefused,
                "Can't Connect To Plug".to_string(),
            )),
        }
    }

    pub fn switch(&mut self) -> Result<Value, Error> {
        match self.status {
            true => self.turn_off(),
            false => self.turn_on(),
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::models::relays::KasaMultiPlug;

    #[test]
    fn test_multiplug_stuff() {
        let ip = "192.168.0.218".to_string();

        let mut plugs: Vec<KasaMultiPlug> = KasaMultiPlug::new(
            ip,
            vec![
                "BedframeLight".parse().unwrap(),
                "BedroomLight".parse().unwrap(),
            ],
            "Bedroom".parse().unwrap(),
        );
        assert_eq!(plugs.len(), 2);

        plugs
            .get_mut(0)
            .unwrap()
            .turn_on()
            .expect("TODO: panic message");

        // if children.is_array() {
        //     println!("{:?}", children.get(0));
        //     for child in children.as_array() {
        //         println!("Here");
        //         println!("Child: {:?}", &child);
        //     }
        // }
        //
        //
        // let parsed_childred = children.as_array().iter().filter_map(|child| {
        //     println!("Child: {:?}", &child);
        //     None::<String>
        // });
    }
}
