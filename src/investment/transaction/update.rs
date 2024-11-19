use super::{has_permission, validate_input};
use crate::database::{get_connection, Transaction};
use crate::error::ServerError;
use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Request {
    token: String,
    transaction: Transaction,
}

#[post("/api/investment/transaction/update")]
pub async fn handler(
    request: web::Json<Request>,
) -> Result<impl Responder, ServerError> {
    let mut conn = get_connection()?;
    let tran = conn.transaction()?;

    let transaction =
        match Transaction::by_id(request.transaction.id, &tran)? {
            None => {
                return Ok(HttpResponse::BadRequest()
                    .body("transaction does not exist"))
            }
            Some(t) => t,
        };

    // permission check
    if !has_permission(&transaction, &request.token, &tran)? {
        return Ok(HttpResponse::Forbidden().finish());
    }

    // input check
    if request.transaction.account != transaction.account {
        return Ok(
            HttpResponse::BadRequest().body("account cannot be modified")
        );
    } else if let Some(err) =
        validate_input(&request.transaction, &tran)
    {
        return Ok(HttpResponse::BadRequest().body(err));
    }

    request.transaction.update(&tran)?;
    tran.commit()?;
    Ok(HttpResponse::Ok().finish())
}
