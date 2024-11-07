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

#[post("/api/investment/account/update")]
pub async fn handler(
    request: web::Json<Request>,
) -> Result<impl Responder, ServerError> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let account = match Account::by_id(request.account.id)? {
        None => {
            return Ok(HttpResponse::BadRequest().body("account does not exist"))
        }
        Some(a) => a,
    };

    // permission check
    let user_id = match authenticate(&request.token, now) {
        Some(i) => i,
        None => return Ok(HttpResponse::Forbidden().finish()),
    };
    if account.owner != user_id {
        return Ok(HttpResponse::Forbidden().finish());
    }

    // input check
    if request.account.name.len() < 4 {
        return Ok(HttpResponse::BadRequest().body("account name too short"));
    } else if request.account.alias.len() < 4 {
        return Ok(HttpResponse::BadRequest().body("account alias too short"));
    } else if request.account.owner != account.owner {
        return Ok(
            HttpResponse::BadRequest().body("owner cannot be modified")
        );
    }

    request.account.update()?;
    Ok(HttpResponse::Ok().finish())
}
