use super::has_permission;
use crate::database::Account;
use crate::error::ServerError;
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
    let account = match Account::by_id(request.account_id)? {
        None => {
            return Ok(HttpResponse::BadRequest().body("account does not exist"))
        }
        Some(a) => a,
    };

    if !has_permission(&account, &request.token)? {
        return Ok(HttpResponse::Forbidden().finish());
    }

    Account::delete(account.id)?;
    Ok(HttpResponse::Ok().finish())
}
