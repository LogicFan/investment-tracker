use super::PRIVATE_KEY;
use crate::database::{get_connection, User};
use crate::error::ServerError;
use crate::user::{AsUser, Claims};
use actix_web::{post, web, HttpResponse, Responder};
use jwt::SignWithKey;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Deserialize)]
struct RequestData {
    token: String,
}

#[derive(Debug, Serialize)]
struct ResponseData {
    username: String,
    token: String,
}

#[post("/api/user/rotate")]
pub async fn handler(
    request: web::Json<RequestData>,
) -> Result<impl Responder, ServerError> {
    let mut conn = get_connection()?;
    let tran = conn.transaction()?;

    let id = match request.token.user_id()? {
        None => return Ok(HttpResponse::Forbidden().finish()),
        Some(i) => i,
    };
    let token = {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let claims = Claims {
            iss: id,
            iat: now,
            exp: now + 3600,
        };
        claims.sign_with_key(&*PRIVATE_KEY)?
    };

    let user = match User::by_id(id, &tran)? {
        None => return Ok(HttpResponse::BadRequest().finish()),
        Some(u) => u,
    };

    let response = ResponseData {
        username: user.username,
        token,
    };
    Ok(HttpResponse::Ok().json(response))
}
