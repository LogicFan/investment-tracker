use super::validate_input;
use crate::database::{connection, Account, Transaction};
use crate::error::ServerError;
use crate::user::authenticate;
use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Request {
    token: String,
    transaction: Transaction,
}

#[post("/api/investment/transaction/insert")]
pub async fn handler(
    request: web::Json<Request>,
) -> Result<impl Responder, ServerError> {
    let mut connection = connection()?;

    let account =
        match Account::by_id(request.transaction.account, &mut connection)? {
            None => {
                return Ok(
                    HttpResponse::BadRequest().body("account does not exist")
                )
            }
            Some(a) => a,
        };

    // permission check
    match authenticate(&request.token)? {
        Some(user) if account.owner == user => (),
        _ => return Ok(HttpResponse::Forbidden().finish()),
    };

    if !request.transaction.id.is_nil() {
        return Ok(
            HttpResponse::BadRequest().body("transaction id should be nil")
        );
    } else if let Some(err) =
        validate_input(&request.transaction, &mut connection)
    {
        return Ok(HttpResponse::BadRequest().body(err));
    }

    request.transaction.insert(&mut connection)?;
    Ok(HttpResponse::Ok().finish())
}
