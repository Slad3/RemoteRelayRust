#![allow(dead_code, non_snake_case)]
use rocket::serde::{Deserialize, Serialize};

macro_rules! pub_struct {
    ($name:ident {$($field:ident: $t:ty,)*}) => {
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        pub struct $name {
            $(pub $field: $t),*
        }
    }
}

pub_struct!(MultiPlugSystemInfo {
    sw_ver: String,
    hw_ver: String,
    model: String,
    deviceId: String,
    oemId: String,
    hwId: String,
    rssi: i32,
    latitude_i: i32,
    longitude_i: i32,
    alias: String,
    status: String,
    obd_src: String,
    mic_type: String,
    feature: String,
    mac: String,
    updating: i32,
    led_off: i32,
    children: Vec<MultiPlugChild>,
    child_num: i32,
    ntc_state: i32,
    err_code: i32,
});

pub_struct!(MultiPlugChild {
    id: String,
    state: i32,
    alias: String,
    on_time: i32,
    next_action: NextAction,
});

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NextAction {
    #[serde(rename = "type")]
    kind: i32,
}

pub_struct!(MultiPlugGetSysInfo {
    get_sysinfo: MultiPlugSystemInfo,
});

pub_struct!(MultiPlugStatus {
    system: MultiPlugGetSysInfo,
});

pub_struct!(PlugSystemInfo {
    sw_ver: String,
    hw_ver: String,
    model: String,
    deviceId: String,
    oemId: String,
    hwId: String,
    rssi: i32,
    latitude_i: i32,
    longitude_i: i32,
    alias: String,
    status: String,
    obd_src: String,
    mic_type: String,
    feature: String,
    mac: String,
    updating: i32,
    led_off: i32,
    relay_state: i32,
    on_time: i32,
    icon_hash: String,
    dev_name: String,
    active_mode: String,
    err_code: i32,
});

pub_struct!(PlugGetSystemInfo {
    get_sysinfo: PlugSystemInfo,
});

pub_struct!(PlugStatus {
    system: PlugGetSystemInfo,
});

pub_struct!(PlugMutateResponse {
    system: PlugMutateSystem,
});

pub_struct!(PlugMutateSystem {
    set_relay_state: ErrCode,
});

#[rustfmt::skip]
pub_struct!(ErrCode {
    err_code: i32,
});
