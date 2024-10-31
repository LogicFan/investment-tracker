use super::PRIVATE_KEY;
use crate::database;
use crate::error::ServerError;
use crate::user::{authenticate, Claims};
use actix_web::{post, web, HttpResponse, Responder};
use jwt::SignWithKey;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Deserialize)]
struct Request {
    token: String,
}

#[derive(Debug, Serialize)]
struct Response {
    username: String,
    token: String,
}

#[post("/api/user/rotate")]
pub async fn handler(
    request: web::Json<Request>,
) -> Result<impl Responder, ServerError> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

    // permission check
    let id = match authenticate(&request.token, now) {
        None => return Ok(HttpResponse::Forbidden().finish()),
        Some(i) => i,
    };
    let user = match database::User::select(id)? {
        None => return Ok(HttpResponse::BadRequest().finish()),
        Some(u) => u,
    };

    let claims = Claims {
        iss: id,
        iat: now,
        exp: now + 3600,
    };
    let token = claims.sign_with_key(&*PRIVATE_KEY)?;

    let response = Response {
        username: user.username,
        token,
    };
    Ok(HttpResponse::Ok().json(response))
}
