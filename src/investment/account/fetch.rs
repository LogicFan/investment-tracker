use crate::{database::Account, error::ServerError};
use crate::user::authenticate;
use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Request {
    token: String,
}

#[post("/api/investment/account/fetch")]
pub async fn handler(
    request: web::Json<Request>,
) -> Result<impl Responder, ServerError> {
    let user_id = match authenticate(&request.token)? {
        None => return Ok(HttpResponse::Forbidden().finish()),
        Some(i) => i
    };

    let accounts = Account::by_owner(user_id)?;
    Ok(HttpResponse::Ok().json(accounts))
}
