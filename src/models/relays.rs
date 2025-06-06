use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Debug;

use rocket::serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use std::io::{Error, ErrorKind};

use crate::models::kasa_network_models::{MultiPlugStatus, PlugMutateResponse, PlugStatus};
use crate::utils::kasa_plug_network_functions;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum RelayType {
    KasaPlug(KasaPlug),
    KasaMultiPlug(KasaMultiPlug),
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct KasaPlug {
    pub(crate) ip: String,
    pub(crate) name: String,
    pub(crate) status: bool,
    pub(crate) room: String,
    pub(crate) tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct KasaMultiPlug {
    pub(crate) ip: String,
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) status: bool,
    pub(crate) room: String,
    pub(crate) tags: Vec<String>,
}

pub trait RelayActions<'a>: Debug + Deserialize<'a> + Serialize {
    fn connected(&mut self) -> Result<bool, Error>;

    fn to_json(&self) -> Value;

    fn get_status(&mut self) -> Result<bool, Error>;

    fn turn_off(&mut self) -> Result<Value, Error>;

    fn turn_on(&mut self) -> Result<Value, Error>;

    fn switch(&mut self) -> Result<Value, Error>;
}

impl KasaPlug {
    pub fn new(ip: String, name: String, room: String, tags: Vec<String>) -> Self {
        KasaPlug {
            ip,
            name,
            status: false,
            tags,
            room,
        }
    }
}

impl RelayActions<'_> for KasaPlug {
    fn connected(&mut self) -> Result<bool, Error> {
        self.get_status()
    }

    fn to_json(&self) -> Value {
        json!({
            "type": "Kasa Plug",
            "ip": &self.ip,
            "name": &self.name,
            "status": &self.status,
            "room": &self.room,
            "tags": &self.tags,
        })
    }

    fn get_status(&mut self) -> Result<bool, Error> {
        let cmd = json!({"system": {"get_sysinfo": {}}});
        let response = kasa_plug_network_functions::send::<PlugStatus>(&self.ip, &cmd.to_string())?;
        let relay_state = response.system.get_sysinfo.relay_state == 1;
        self.status = relay_state;
        Ok(relay_state)
    }

    fn turn_off(&mut self) -> Result<Value, Error> {
        let cmd = json!({"system": {"set_relay_state": {"state": 0}}});

        match kasa_plug_network_functions::send::<PlugMutateResponse>(&self.ip, &cmd.to_string()) {
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

    fn turn_on(&mut self) -> Result<Value, Error> {
        let cmd = json!({"system": {"set_relay_state": {"state": 1}}});
        match kasa_plug_network_functions::send::<PlugMutateResponse>(&self.ip, &cmd.to_string()) {
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

    fn switch(&mut self) -> Result<Value, Error> {
        match self.status {
            true => self.turn_off(),
            false => self.turn_on(),
        }
    }
}

impl KasaMultiPlug {
    pub fn new(
        ip: String,
        names: Vec<String>,
        room: String,
        tags: Vec<String>,
    ) -> Result<Vec<KasaMultiPlug>, Error> {
        let command = json!({"system": {"get_sysinfo": {}}});
        let response =
            match kasa_plug_network_functions::send::<MultiPlugStatus>(&ip, &command.to_string()) {
                Ok(response) => response,
                Err(..) => {
                    return Err(Error::new(
                        ErrorKind::NotConnected,
                        format!("Unable to connect to KasaMultiPlug {}", ip),
                    ))
                }
            };

        let mut multi_plug_children: Vec<KasaMultiPlug> = Vec::new();

        for (child, name) in response
            .system
            .get_sysinfo
            .children
            .iter()
            .zip(names.iter())
        {
            multi_plug_children.push(KasaMultiPlug {
                ip: ip.clone(),
                id: child.id.to_string(),
                name: name.clone(),
                status: child.state == 1,
                room: room.clone(),
                tags: tags.clone(),
            })
        }

        Ok(multi_plug_children)
    }
}

impl RelayActions<'_> for KasaMultiPlug {
    fn connected(&mut self) -> Result<bool, Error> {
        let command = json!({"system": {"get_sysinfo": {}}});
        let _ =
            kasa_plug_network_functions::send::<MultiPlugStatus>(&self.ip, &command.to_string())?;
        Ok(true)
    }

    fn to_json(&self) -> Value {
        json!({
            "type": "Kasa Plug",
            "ip": &self.ip,
            "id": &self.id,
            "name": &self.name,
            "status": self.status,
            "room": &self.room,
            "tags": &self.tags,
        })
    }

    fn get_status(&mut self) -> Result<bool, Error> {
        let cmd = json!({"system": {"get_sysinfo": {}}});
        let result =
            kasa_plug_network_functions::send::<MultiPlugStatus>(&self.ip, &cmd.to_string())?;

        for child in result.system.get_sysinfo.children {
            if child.id == self.id {
                let relay_state = child.state == 1;
                self.status = relay_state;
                return Ok(relay_state);
            }
        }

        Err(Error::new(
            ErrorKind::NotFound,
            "Can't Connect to To Plug".to_string(),
        ))
    }

    fn turn_off(&mut self) -> Result<Value, Error> {
        let cmd = json!({"context": {"child_ids": [self.id.clone()]}, "system": {"set_relay_state": {"state": 0}}});
        match kasa_plug_network_functions::send::<PlugMutateResponse>(&self.ip, &cmd.to_string()) {
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

    fn turn_on(&mut self) -> Result<Value, Error> {
        let cmd = json!({"context": {"child_ids": [self.id.clone()]}, "system": {"set_relay_state": {"state": 1}}});
        match kasa_plug_network_functions::send::<PlugMutateResponse>(&self.ip, &cmd.to_string()) {
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

    fn switch(&mut self) -> Result<Value, Error> {
        match self.status {
            true => self.turn_off(),
            false => self.turn_on(),
        }
    }
}

impl RelayActions<'_> for RelayType {
    fn connected(&mut self) -> Result<bool, Error> {
        match self {
            RelayType::KasaPlug(relay_plug) => relay_plug.connected(),
            RelayType::KasaMultiPlug(relay_plug) => relay_plug.connected(),
        }
    }

    fn to_json(&self) -> Value {
        match self {
            RelayType::KasaPlug(relay_plug) => relay_plug.to_json(),
            RelayType::KasaMultiPlug(relay_plug) => relay_plug.to_json(),
        }
    }

    fn get_status(&mut self) -> Result<bool, Error> {
        match self {
            RelayType::KasaPlug(relay_plug) => relay_plug.get_status(),
            RelayType::KasaMultiPlug(relay_plug) => relay_plug.get_status(),
        }
    }

    fn turn_off(&mut self) -> Result<Value, Error> {
        match self {
            RelayType::KasaPlug(relay_plug) => relay_plug.turn_off(),
            RelayType::KasaMultiPlug(relay_plug) => relay_plug.turn_off(),
        }
    }

    fn turn_on(&mut self) -> Result<Value, Error> {
        match self {
            RelayType::KasaPlug(relay_plug) => relay_plug.turn_on(),
            RelayType::KasaMultiPlug(relay_plug) => relay_plug.turn_on(),
        }
    }

    fn switch(&mut self) -> Result<Value, Error> {
        match self {
            RelayType::KasaPlug(relay_plug) => relay_plug.switch(),
            RelayType::KasaMultiPlug(relay_plug) => relay_plug.switch(),
        }
    }
}

pub fn config_equals<T>(map1: &HashMap<String, T>, map2: &HashMap<String, T>) -> bool
where
    T: PartialEq,
{
    if map1.len() != map2.len() {
        return false;
    }

    for (key, value1) in map1 {
        match map2.get(key) {
            Some(value2) => {
                if value1 != value2 {
                    return false;
                }
            }
            None => return false,
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use crate::models::relays::{KasaMultiPlug, RelayActions};

    // #[test]
    // fn test_singleplug_timeouts() {
    //     let mut plug = KasaPlug::new("192.168.0.107".to_string(), "ScentLight".to_string(), "bedroom".to_string());
    //     println!("{:?}", &plug.connected());
    // }

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
            vec![],
        )
        .unwrap();
        assert_eq!(plugs.len(), 2);

        for plug in &plugs {
            println!("{} \t {}", &plug.name, &plug.id);
        }

        plugs
            .get_mut(0)
            .unwrap()
            .turn_on()
            .expect("TODO: panic message");
    }
}
