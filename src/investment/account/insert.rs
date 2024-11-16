use crate::database::{connection, Account};
use crate::error::ServerError;
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
    let mut connection = connection()?;

    if !request
        .account
        .has_permission(&request.token, &mut connection)?
    {
        return Ok(HttpResponse::Forbidden().finish());
    }

    // input check
    if !request.account.id.is_nil() {
        return Ok(HttpResponse::BadRequest().body("account id should be nil"));
    } else if let Some(err) = request.account.validate_input(&mut connection) {
        return Ok(HttpResponse::BadRequest().body(err));
    }

    request.account.insert(&mut connection)?;
    Ok(HttpResponse::Ok().finish())
}
