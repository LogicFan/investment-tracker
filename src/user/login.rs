use super::PRIVATE_KEY;
use crate::{database::User, error::ServerError};
use crate::user::Claims;
use actix_web::{post, web, HttpResponse, Responder};
use jwt::SignWithKey;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Deserialize)]
struct Request {
    username: String,
    password: String,
}

#[derive(Debug, Serialize)]
struct Response {
    username: String,
    token: String,
}

// TODO: set retry interval after 3 attempts.
#[post("/api/user/login")]
pub async fn handler(
    request: web::Json<Request>,
) -> Result<impl Responder, ServerError> {
    // permission check
    let user =
        match User::by_username(request.username.clone())? {
            None => return Ok(HttpResponse::BadRequest().finish()),
            Some(u) => u,
        };
    if Sha256::digest(request.password.clone()).to_vec() != user.password {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let claims = Claims {
        iss: user.id,
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
