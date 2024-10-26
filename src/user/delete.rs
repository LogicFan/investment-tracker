use crate::database;
use crate::error::ServerError;
use crate::user::authenticate;
use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct Request {
    token: String,
    id: Uuid,
}

#[post("/api/user/delete")]
pub async fn handler(
    request: web::Json<Request>,
) -> Result<impl Responder, ServerError> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

    // permission check
    let id = match authenticate(&request.token, now) {
        None => return Ok(HttpResponse::Forbidden().finish()),
        Some(i) => i,
    };
    if id != request.id {
        return Ok(HttpResponse::Forbidden().finish());
    }

    database::users::delete(request.id)?;
    Ok(HttpResponse::Ok().finish())
}
