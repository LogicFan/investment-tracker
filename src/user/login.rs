use super::PRIVATE_KEY;
use crate::database::{connection, User};
use crate::error::ServerError;
use crate::user::Claims;
use actix_web::{post, web, HttpResponse, Responder};
use jwt::SignWithKey;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Deserialize)]
struct RequestData {
    username: String,
    password: String,
}

#[derive(Debug, Serialize)]
struct ResponseData {
    username: String,
    token: String,
}

#[post("/api/user/login")]
pub async fn handler(
    request: web::Json<RequestData>,
) -> Result<impl Responder, ServerError> {
    let mut connection = connection()?;

    let user =
        match User::by_username(request.username.clone(), &mut connection)? {
            None => {
                return Ok(HttpResponse::BadRequest().body("unknown username"))
            }
            Some(u) => u,
        };

    if user.attempts(&mut connection)? >= 3 {
        return Ok(HttpResponse::Forbidden().body("try again after 1 minute"));
    }

    if Sha256::digest(request.password.clone()).to_vec() != user.password {
        user.add_attempt(&mut connection)?;
        return Ok(HttpResponse::Forbidden().body("incorrect password"));
    }

    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let claims = Claims {
        iss: user.id,
        iat: now,
        exp: now + 3600,
    };
    let token = claims.sign_with_key(&*PRIVATE_KEY)?;
    let response = ResponseData {
        username: user.username,
        token,
    };
    Ok(HttpResponse::Ok().json(response))
}
