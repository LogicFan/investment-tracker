use crate::database;
use crate::database::users::User;
use crate::error::ServerError;
use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct Request {
    username: String,
    password: String,
}

#[post("/api/user/register")]
pub async fn handler(
    request: web::Json<Request>,
) -> Result<impl Responder, ServerError> {
    // input check
    if request.username.len() < 6 {
        return Ok(HttpResponse::BadRequest().body("username too short"));
    } else if request.password.len() < 8 {
        return Ok(HttpResponse::BadRequest().body("password too short"));
    }

    database::users::insert(User {
        id: Uuid::nil(),
        username: request.username.clone(),
        password: Sha256::digest(request.password.clone()).to_vec(),
    })?;

    Ok(HttpResponse::Ok().finish())
}
