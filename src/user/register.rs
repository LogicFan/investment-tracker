use crate::database::{connection, User};
use crate::error::ServerError;
use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;
use sha2::{Digest, Sha256};

#[derive(Debug, Deserialize)]
struct RequestData {
    username: String,
    password: String,
}

#[post("/api/user/register")]
pub async fn handler(
    request: web::Json<RequestData>,
) -> Result<impl Responder, ServerError> {
    let mut connection = connection()?;

    // input check
    if request.username.len() < 6 {
        return Ok(HttpResponse::BadRequest().body("username too short"));
    } else if request.password.len() < 8 {
        return Ok(HttpResponse::BadRequest().body("password too short"));
    }

    User::new(
        request.username.clone(),
        Sha256::digest(request.password.clone()).to_vec(),
    )
    .insert(&mut connection)?;

    Ok(HttpResponse::Ok().finish())
}
