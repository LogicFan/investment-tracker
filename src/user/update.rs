use crate::database::User;
use crate::error::ServerError;
use crate::user::authenticate;
use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;
use sha2::{Digest, Sha256};

#[derive(Debug, Deserialize)]
struct RequestData {
    token: String,
    username: Option<String>,
    password: Option<(String, String)>,
}

#[post("/api/user/update")]
pub async fn handler(
    request: web::Json<RequestData>,
) -> Result<impl Responder, ServerError> {
    // permission check
    let id = match authenticate(&request.token)? {
        None => return Ok(HttpResponse::Forbidden().finish()),
        Some(i) => i,
    };
    let mut user = match User::by_id(id)? {
        None => return Ok(HttpResponse::BadRequest().finish()),
        Some(u) => u,
    };

    // update values
    if let Some(username) = request.username.clone() {
        user.username = username
    }
    if let Some((old_password, new_password)) = request.password.clone() {
        // another permission check
        if user.password != Sha256::digest(old_password).to_vec() {
            return Ok(HttpResponse::Forbidden().finish());
        }
        user.password = Sha256::digest(new_password).to_vec()
    }

    user.update()?;
    Ok(HttpResponse::Ok().finish())
}
