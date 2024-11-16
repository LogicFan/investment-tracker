use crate::database::{connection, Account};
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
    let mut connection = connection()?;

    let account = match Account::by_id(request.account.id, &mut connection)? {
        None => {
            return Ok(HttpResponse::BadRequest().body("account does not exist"))
        }
        Some(a) => a,
    };

    if !authenticate(&account, &request.token, &mut connection)? {
        return Ok(HttpResponse::Forbidden().finish());
    }

    if request.account.owner != account.owner {
        return Ok(HttpResponse::BadRequest().body("owner cannot be modified"));
    } else if let Some(err) = validate(&request.account, &mut connection)
    {
        return Ok(HttpResponse::BadRequest().body(err));
    }

    request.account.update(&mut connection)?;
    Ok(HttpResponse::Ok().finish())
}
