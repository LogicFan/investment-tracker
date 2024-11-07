use super::{has_permission, validate_input};
use crate::database::Account;
use crate::error::ServerError;
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
    let account = match Account::by_id(request.account.id)? {
        None => {
            return Ok(HttpResponse::BadRequest().body("account does not exist"))
        }
        Some(a) => a,
    };

    if !has_permission(&account, &request.token)? {
        return Ok(HttpResponse::Forbidden().finish());
    }

    if request.account.owner != account.owner {
        return Ok(HttpResponse::BadRequest().body("owner cannot be modified"));
    } else if let Some(err) = validate_input(&request.account) {
        return Ok(HttpResponse::BadRequest().body(err));
    }

    request.account.update()?;
    Ok(HttpResponse::Ok().finish())
}
