use crate::database::{Account, Transaction};
use crate::error::ServerError;
use crate::user::authenticate;
use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Deserialize)]
struct Request {
    token: String,
    transaction: Transaction,
}

#[post("/api/investment/transaction/update")]
pub async fn handler(
    request: web::Json<Request>,
) -> Result<impl Responder, ServerError> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let transaction = match Transaction::by_id(request.transaction.id)? {
        None => {
            return Ok(
                HttpResponse::BadRequest().body("transaction does not exist")
            )
        }
        Some(t) => t,
    };
    let account = match Account::by_id(transaction.account)? {
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
    if request.transaction.account != transaction.account {
        return Ok(
            HttpResponse::BadRequest().body("account cannot be modified")
        );
    }

    request.transaction.update()?;
    Ok(HttpResponse::Ok().finish())
}
