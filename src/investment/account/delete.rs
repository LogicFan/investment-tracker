use crate::database::{get_connection, Account};
use crate::error::ServerError;
use crate::investment::account::authenticate;
use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;
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
    let mut conn = get_connection()?;
    let tran = conn.transaction()?;

    let account = match Account::by_id(request.account_id, &tran)? {
        None => {
            return Ok(HttpResponse::BadRequest().body("account does not exist"))
        }
        Some(a) => a,
    };

    if !authenticate(&account, &request.token, &tran)? {
        return Ok(HttpResponse::Forbidden().finish());
    }

    Account::delete(account.id, &tran)?;
    tran.commit()?;
    Ok(HttpResponse::Ok().finish())
}
