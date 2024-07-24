use rocket::http::uri::fmt::UriQueryArgument::Raw;
use rocket::response::content::{RawJson, RawText};
use crate::RELAYS;

#[get("/external")]
pub fn external_route() -> RawText<String> {

    RawText("External Route".to_string())
}