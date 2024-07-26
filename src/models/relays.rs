use serde_json::Value;

use rocket::time::Error;
use serde::Serialize;
use serde_json::json;
use std::convert::TryFrom;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::vec;
use rocket::serde::Deserialize;

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
        let mut tags: Vec<String> = Vec::new();
        let mut plug = KasaPlug {
            ip,
            name,
            status: false,
            tags,
            room,
        };
        // plug.status = KasaPlug::get_status(&mut plug);
        plug
    }

    pub fn new_static(ip: &str, name: &str, room: &str) -> Self {
        KasaPlug::new(ip.to_string(), name.to_string(), room.to_string())
    }

    pub fn send(&self, cmd: String) -> Value {
        const PORT: u16 = 9999;
        let timeout = 10;
        let mut stream = TcpStream::connect((self.ip.to_string(), PORT)).unwrap();
        let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(timeout)));
        let encrypted = KasaPlug::encrypt(cmd);
        stream.write_all(&encrypted).unwrap();
        let mut data = vec![0; 4096];
        stream.read(&mut data).unwrap();

        let a_ref: &[u8] = &data[..4];
        let b = i32::from_be_bytes(<[u8; 4]>::try_from(a_ref).expect("Unable to parse bytes"));

        let end_pos: i32 = b + 4 as i32;

        let decrypted = KasaPlug::decrypt(data[4..end_pos as usize].to_vec());
        let json_data: Value = serde_json::from_str(&decrypted.as_str()).unwrap();
        return json_data;
    }

    pub fn get_info(&self) -> Value {
        let cmd = json!({"system": {"get_sysinfo": {}}});
        self.send(cmd.to_string())["system"]["get_sysinfo"].clone()
    }

    pub fn wlan_scan(&self) -> Value {
        let cmd = json!({"netif": {"get_scaninfo": {"refresh": 0}}});
        self.send(cmd.to_string())
    }

    pub(crate) fn connected(&mut self) -> Result<bool, Error> {
        Ok(self.get_status())
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

    pub fn get_status(&mut self) -> bool {
        let cmd = json!({"system": {"get_sysinfo": {}}});
        let result = self.send(cmd.to_string());
        let relay_state = result["system"]["get_sysinfo"]["relay_state"]
            .as_u64()
            .unwrap_or(0)
            == 1;
        self.status = relay_state;
        relay_state
    }

    pub fn turn_off(&mut self) {
        let cmd = json!({"system": {"set_relay_state": {"state": 0}}});
        let _ = self.send(cmd.to_string())["system"].clone();
        self.status = false;
    }

    pub fn turn_on(&mut self) {
        let cmd = json!({"system": {"set_relay_state": {"state": 1}}});
        let _ = self.send(cmd.to_string())["system"].clone();
        self.status = true;
    }

    pub fn switch(&mut self) {
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
