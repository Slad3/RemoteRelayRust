use rocket::http::{ContentType, Status};
use rocket::serde::json::Json;
use serde_json::Value;

use rocket::request::Request;
use rocket::response;
use rocket::response::{Responder, Response};

#[derive(Debug)]
pub(crate) struct ApiResponse {
    pub(crate) value: Json<Value>,
    pub(crate) status: Status,
}

impl<'r> Responder<'r, 'r> for ApiResponse {
    fn respond_to(self, req: &Request) -> response::Result<'r> {
        Response::build_from(self.value.respond_to(&req)?)
            .status(self.status)
            .header(ContentType::JSON)
            .ok()
    }
}
