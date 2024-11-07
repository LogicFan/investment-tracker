use crate::database::Account;
use crate::error::ServerError;
use crate::user::authenticate;
use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct Request {
    #[allow(unused)]
    token: String,
    account_id: Uuid,
}

#[post("/api/investment/account/delete")]
pub async fn handler(
    request: web::Json<Request>,
) -> Result<impl Responder, ServerError> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let account = match Account::by_id(request.account_id)? {
        None => {
            return Ok(HttpResponse::BadRequest().body("account does not exist"))
        }
        Some(a) => a,
    };

    // permission check
    let user_id = match authenticate(&request.token, now) {
        None => return Ok(HttpResponse::Forbidden().finish()),
        Some(i) => i,
    };
    if user_id != account.owner {
        return Ok(HttpResponse::Forbidden().finish());
    }

    account.delete()?;
    Ok(HttpResponse::Ok().finish())
}
