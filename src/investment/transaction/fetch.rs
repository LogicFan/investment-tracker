use crate::database::{Account, Transaction};
use crate::error::ServerError;
use crate::user::authenticate;
use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct Request {
    token: String,
    accounts: Vec<Uuid>,
}

#[post("/api/investment/transaction/fetch")]
pub async fn handler(
    request: web::Json<Request>,
) -> Result<impl Responder, ServerError> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

    // permission check
    let user_id = match authenticate(&request.token, now) {
        None => return Ok(HttpResponse::Forbidden().finish()),
        Some(i) => i,
    };
    for account_id in &request.accounts {
        let account = match Account::by_id(*account_id)? {
            None => {
                return Ok(
                    HttpResponse::BadRequest().body("account does not exist")
                )
            }
            Some(a) => a,
        };
        if account.owner != user_id {
            return Ok(HttpResponse::Forbidden().finish());
        }
    }

    let transactions = Transaction::by_accounts(request.accounts.clone())?;
    Ok(HttpResponse::Ok().json(transactions))
}
