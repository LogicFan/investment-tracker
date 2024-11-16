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

#[post("/api/investment/account/insert")]
pub async fn handler(
    request: web::Json<Request>,
) -> Result<impl Responder, ServerError> {
    let mut connection = get_connection()?;

    if !authenticate(&request.account, &request.token, &mut connection)? {
        return Ok(HttpResponse::Forbidden().finish());
    }

    // input check
    if !request.account.id.is_nil() {
        return Ok(HttpResponse::BadRequest().body("account id should be nil"));
    } else if let Some(err) = validate(&request.account, &mut connection)
    {
        return Ok(HttpResponse::BadRequest().body(err));
    }

    request.account.insert(&mut connection)?;
    Ok(HttpResponse::Ok().finish())
}
