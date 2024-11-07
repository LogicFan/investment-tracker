use super::{validate_input, has_permission};
use crate::database::Account;
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
    if !has_permission(&request.account, &request.token)? {
        return Ok(HttpResponse::Forbidden().finish());
    }

    // input check
    if !request.account.id.is_nil() {
        return Ok(HttpResponse::BadRequest().body("account id should be nil"));
    } else if let Some(err) = validate_input(&request.account) {
        return Ok(HttpResponse::BadRequest().body(err));
    }

    request.account.insert()?;
    Ok(HttpResponse::Ok().finish())
}
