#![allow(dead_code, non_snake_case)]
use serde_json::json;
use serde_json::Value;
use std::convert::TryFrom;
use std::io::{Error, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::vec;

pub fn decrypt(string: Vec<u8>) -> String {
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

pub fn encrypt(string: &String) -> Vec<u8> {
    let mut key: u8 = 171;
    let mut result: Vec<u8> = vec![];
    result.extend_from_slice(&(string.len() as u32).to_be_bytes());
    for i in string.bytes() {
        let a = key ^ i;
        key = a;

        result.push(a);
    }
    result
}

pub fn send<T: serde::de::DeserializeOwned>(ip: &String, cmd: &String) -> Result<T, Error> {
    const PORT: u16 = 9999;
    let timeout = 1;
    let mut stream = TcpStream::connect((ip.clone(), PORT))?;
    let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(timeout)));
    let encrypted = encrypt(cmd);
    stream.write_all(&encrypted)?;
    let mut data = vec![0; 4096];
    stream.read(&mut data)?;

    let a_ref: &[u8] = &data[..4];
    let b = i32::from_be_bytes(<[u8; 4]>::try_from(a_ref).expect("Unable to parse bytes"));

    let end_pos: i32 = b + 4i32;

    let decrypted = decrypt(data[4..end_pos as usize].to_vec());
    let json_data: T = serde_json::from_str::<T>(&decrypted.as_str()).expect("Cannot parse");
    Ok(json_data)
}

pub fn get_info<T: serde::de::DeserializeOwned>(ip: String) -> Result<T, Error> {
    let cmd = json!({"system": {"get_sysinfo": {}}});
    match send::<T>(&ip, &cmd.to_string()) {
        Ok(result) => Ok(result),
        Err(..) => Err(Error::new(
            ErrorKind::ConnectionRefused,
            "Can't Connect To Plug".to_string(),
        )),
    }
}

pub fn wlan_scan(ip: String) -> Result<Value, Error> {
    let cmd = json!({"netif": {"get_scaninfo": {"refresh": 0}}});
    send::<Value>(&ip, &cmd.to_string())
}
