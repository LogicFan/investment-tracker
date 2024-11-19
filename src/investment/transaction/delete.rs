use super::has_permission;
use crate::database::{get_connection, Transaction};
use crate::error::ServerError;
use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct Request {
    #[allow(unused)]
    token: String,
    transaction_id: Uuid,
}

#[post("/api/investment/transaction/delete")]
pub async fn handler(
    request: web::Json<Request>,
) -> Result<impl Responder, ServerError> {
    let mut conn = get_connection()?;
    let tran = conn.transaction()?;

    let transaction =
        match Transaction::by_id(request.transaction_id, &tran)? {
            None => {
                return Ok(HttpResponse::BadRequest()
                    .body("transaction does not exist"))
            }
            Some(t) => t,
        };

    if !has_permission(&transaction, &request.token, &tran)? {
        return Ok(HttpResponse::Forbidden().finish());
    }

    Transaction::delete(transaction.id, &tran)?;
    Ok(HttpResponse::Ok().finish())
}
