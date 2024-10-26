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
    let user_id = authenticate(&request.token, now);
    if user_id.is_none() {
        return Ok(HttpResponse::Forbidden().finish());
    }
    let user_id = user_id.unwrap();

    let user = database::users::select(Some(user_id), None)?;
    if user.is_none() {
        return Ok(HttpResponse::BadRequest().finish());
    }
    let user = user.unwrap();

    let claims = Claims {
        iss: user_id,
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