use crate::database::{get_connection, Account};
use crate::error::ServerError;
use crate::investment::account::{authenticate, validate};
use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Request {
    token: String,
    account: Account,
}

#[post("/api/investment/account/update")]
pub async fn handler(
    request: web::Json<Request>,
) -> Result<impl Responder, ServerError> {
    let mut conn = get_connection()?;
    let tran = conn.transaction()?;

    let account = match Account::by_id(request.account.id, &tran)? {
        None => {
            return Ok(HttpResponse::BadRequest().body("account does not exist"))
        }
        Some(a) => a,
    };

    if !authenticate(&account, &request.token, &tran)? {
        return Ok(HttpResponse::Forbidden().finish());
    }

    if request.account.owner != account.owner {
        return Ok(HttpResponse::BadRequest().body("owner cannot be modified"));
    } else if let Some(err) = validate(&request.account, &tran) {
        return Ok(HttpResponse::BadRequest().body(err));
    }

    request.account.update(&tran)?;
    tran.commit()?;
    Ok(HttpResponse::Ok().finish())
}
