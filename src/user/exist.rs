use crate::database::{get_connection, User};
use crate::error::ServerError;
use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct RequestData {
    username: String,
}

#[post("/api/user/exist")]
pub async fn handler(
    request: web::Json<RequestData>,
) -> Result<impl Responder, ServerError> {
    let mut connection = get_connection()?;

    let has_user =
        User::by_username(request.username.clone(), &mut connection)?.is_some();
    Ok(HttpResponse::Ok().json(has_user))
}
