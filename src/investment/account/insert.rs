use crate::database::Account;
use crate::error::ServerError;
use crate::user::authenticate;
use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Deserialize)]
struct Request {
    token: String,
    account: Account,
}

#[post("/api/investment/account/insert")]
pub async fn handler(
    request: web::Json<Request>,
) -> Result<impl Responder, ServerError> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

    // permission check
    let user_id = match authenticate(&request.token, now) {
        None => return Ok(HttpResponse::Forbidden().finish()),
        Some(i) => i,
    };
    if request.account.owner != user_id {
        return Ok(HttpResponse::Forbidden().finish());
    }

    // input check
    if !request.account.id.is_nil() {
        return Ok(HttpResponse::BadRequest().body("account id should be nil"));
    } else if request.account.name.len() < 4 {
        return Ok(HttpResponse::BadRequest().body("account name too short"));
    } else if request.account.alias.len() < 4 {
        return Ok(HttpResponse::BadRequest().body("account alias too short"));
    }

    request.account.insert()?;
    Ok(HttpResponse::Ok().finish())
}
