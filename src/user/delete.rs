use crate::auth::Authentication;
use crate::database::{get_connection, User};
use crate::error::ServerError;
use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct RequestData {
    token: String,
    password: String,
    id: Uuid,
}

#[post("/api/user/delete")]
pub async fn handler(
    request: web::Json<RequestData>,
) -> Result<impl Responder, ServerError> {
    let mut conn = get_connection()?;
    let tran = conn.transaction()?;

    let user = match User::by_id(request.id, &tran)? {
        None => return Ok(HttpResponse::BadRequest().finish()),
        Some(u) => u,
    };
    if !user.auth(&request.token, &tran)? {
        return Ok(HttpResponse::Forbidden().finish());
    } else if user.password != Sha256::digest(request.password.clone()).to_vec() {
        return Ok(HttpResponse::Forbidden().finish());
    }

    User::delete(user.id, &tran)?;
    tran.commit()?;
    Ok(HttpResponse::Ok().finish())
}
