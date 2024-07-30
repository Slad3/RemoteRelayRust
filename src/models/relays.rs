use serde_json::Value;

use rocket::serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use std::convert::TryFrom;
use std::io::{Error, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::vec;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Relay {
    #[serde(rename = "type")]
    pub(crate) relay_type: String,
    pub(crate) name: String,
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

impl KasaPlug {
    fn decrypt(string: Vec<u8>) -> String {
        let key: u8 = 171;
        let mut result = String::new();
        let mut prev = key;
        for i in string {
            let a = prev ^ i;
            prev = i;
            result.push(a as char);
        }
        result
    }

    fn encrypt(string: String) -> Vec<u8> {
        let mut key: u8 = 171;
        let mut result: Vec<u8> = vec![];
        result.extend_from_slice(&(string.len() as u32).to_be_bytes());
        for i in string.bytes() {
            let a = key ^ i;
            key = a;

            result.push(a);
        }
        return result;
    }

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

    pub fn new_static(ip: &str, name: &str, room: &str) -> Self {
        KasaPlug::new(ip.to_string(), name.to_string(), room.to_string())
    }

    pub fn send(&self, cmd: String) -> Result<Value, Error> {
        const PORT: u16 = 9999;
        let timeout = 10;
        let mut stream = TcpStream::connect((self.ip.to_string(), PORT))?;
        let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(timeout)));
        let encrypted = KasaPlug::encrypt(cmd);
        stream.write_all(&encrypted)?;
        let mut data = vec![0; 4096];
        stream.read(&mut data)?;

        let a_ref: &[u8] = &data[..4];
        let b = i32::from_be_bytes(<[u8; 4]>::try_from(a_ref).expect("Unable to parse bytes"));

        let end_pos: i32 = b + 4i32;

        let decrypted = KasaPlug::decrypt(data[4..end_pos as usize].to_vec());
        let json_data: Value = serde_json::from_str(&decrypted.as_str())?;
        Ok(json_data)
    }

    pub fn get_info(&self) -> Result<Value, Error> {
        let cmd = json!({"system": {"get_sysinfo": {}}});
        let result = self.send(cmd.to_string());
        match result {
            Ok(result) => Ok(result["system"]["get_sysinfo"].clone()),
            Err(..) => Err(Error::new(
                ErrorKind::ConnectionRefused,
                "Can't Connect To Plug".to_string(),
            )),
        }
    }

    pub fn wlan_scan(&self) -> Result<Value, Error> {
        let cmd = json!({"netif": {"get_scaninfo": {"refresh": 0}}});
        self.send(cmd.to_string())
    }

    pub(crate) fn connected(&mut self) -> Result<bool, Error> {
        self.get_status()
    }

    pub fn to_json(&self) -> Value {
        return json!({
            "type": "Kasa Plug",
            "ip": &self.ip.to_string(),
            "name": &self.name.to_string(),
            "status": &self.status,
            "room": &self.room.to_string(),
            "tags": &self.tags,
        });
    }

    pub fn get_status(&mut self) -> Result<bool, Error> {
        let cmd = json!({"system": {"get_sysinfo": {}}});
        let result = self.send(cmd.to_string());

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

    pub fn turn_off(&mut self) -> Result<bool, Error> {
        let cmd = json!({"system": {"set_relay_state": {"state": 0}}});
        let result = self.send(cmd.to_string());
        match result {
            Ok(..) => {
                self.status = false;
                Ok(true)
            }
            Err(..) => Err(Error::new(
                ErrorKind::ConnectionRefused,
                "Can't Connect To Plug".to_string(),
            )),
        }
    }

pub fn turn_on(&mut self) -> Result<bool, Error> {
        let cmd = json!({"system": {"set_relay_state": {"state": 1}}});
        let result = self.send(cmd.to_string());
        match result {
            Ok(..) => {
                self.status = true;
                Ok(true)
            }
            Err(..) => Err(Error::new(
                ErrorKind::ConnectionRefused,
                "Can't Connect To Plug".to_string(),
            )),
        }
    }

    pub fn switch(&mut self) -> Result<bool, Error> {
        return match self.status {
            true => self.turn_off(),
            false => self.turn_on(),
        };
    }
}

impl std::fmt::Display for KasaPlug {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
