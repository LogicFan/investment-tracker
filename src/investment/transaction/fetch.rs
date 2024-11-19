use crate::database::{get_connection, Account, Transaction};
use crate::error::ServerError;
use crate::user::authenticate;
use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct Request {
    token: String,
    account: Uuid,
}

#[post("/api/investment/transaction/fetch")]
pub async fn handler(
    request: web::Json<Request>,
) -> Result<impl Responder, ServerError> {
    let mut conn = get_connection()?;
    let tran = conn.transaction()?;

    let account = match Account::by_id(request.account, &tran)? {
        None => {
            return Ok(HttpResponse::BadRequest().body("account does not exist"))
        }
        Some(a) => a,
    };

    // permission check
    match authenticate(&request.token)? {
        Some(user) if account.owner == user => (),
        _ => return Ok(HttpResponse::Forbidden().finish()),
    };

    let transactions = Transaction::by_account(request.account, &tran)?;
    tran.commit()?;
    Ok(HttpResponse::Ok().json(transactions))
}
