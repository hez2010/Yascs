pub mod message;
pub mod user;

use actix_web::{http::StatusCode, HttpResponse, Responder};
use serde::{Serialize, Deserialize};
use std::future::{ready, Ready};

#[derive(Serialize, Debug)]
pub struct ResultModel<T: Serialize> {
    pub success: bool,
    pub code: u16,
    pub data: Option<T>,
    pub message: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchModel {
    pub patterns: String,
    pub page: Option<i32>,
}

impl<T: Serialize> Responder for ResultModel<T> {
    type Error = serde_json::Error;

    type Future = Ready<Result<HttpResponse, serde_json::Error>>;

    fn respond_to(self, _req: &actix_web::HttpRequest) -> Self::Future {
        let body = serde_json::to_string(&self);

        ready(match body {
            Ok(res) => match StatusCode::from_u16(self.code) {
                Ok(status) => Ok(HttpResponse::build(status)
                    .content_type("application/json")
                    .body(res)),
                Err(err) => Ok(HttpResponse::InternalServerError()
                    .content_type("application/json")
                    .body(format!(
                        "{{\"success\":false,code:500,message:\"{}\"}}",
                        err.to_string()
                    ))),
            },
            Err(err) => Ok(HttpResponse::InternalServerError()
                .content_type("application/json")
                .body(format!(
                    "{{\"success\":false,code:500,message:\"{}\"}}",
                    err.to_string()
                ))),
        })
    }
}
