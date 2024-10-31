use crate::database::User;
use crate::error::ServerError;
use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Request {
    username: String,
}

#[post("/api/user/exist")]
pub async fn handler(
    request: web::Json<Request>,
) -> Result<impl Responder, ServerError> {
    let has_user = User::by_username(request.username.clone())?.is_some();
    Ok(HttpResponse::Ok().json(has_user))
}
