use crate::database::User;
use crate::error::ServerError;
use crate::user::authenticate;
use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct Request {
    token: String,
    password: String,
    id: Uuid,
}

#[post("/api/user/delete")]
pub async fn handler(
    request: web::Json<Request>,
) -> Result<impl Responder, ServerError> {
    let id = match authenticate(&request.token)? {
        None => return Ok(HttpResponse::Forbidden().finish()),
        Some(i) => i,
    };
    if id != request.id {
        return Ok(HttpResponse::Forbidden().finish());
    }
    let user = match User::by_id(request.id)? {
        None => return Ok(HttpResponse::BadRequest().finish()),
        Some(u) => u,
    };
    if user.password != Sha256::digest(request.password.clone()).to_vec() {
        return Ok(HttpResponse::Forbidden().finish());
    }

    user.delete()?;
    Ok(HttpResponse::Ok().finish())
}
