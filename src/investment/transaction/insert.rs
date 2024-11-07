use crate::database::account::AccountKind;
use crate::database::asset::AssetId;
use crate::database::transaction::TxnAction;
use crate::database::{Account, Transaction};
use crate::error::ServerError;
use crate::user::authenticate;
use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Deserialize)]
struct Request {
    token: String,
    transaction: Transaction,
}

#[post("/api/investment/transaction/insert")]
pub async fn handler(
    request: web::Json<Request>,
) -> Result<impl Responder, ServerError> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let account = match Account::by_id(request.transaction.account)? {
        None => {
            return Ok(HttpResponse::BadRequest().body("account does not exist"))
        }
        Some(a) => a,
    };

    // permission check
    let user_id = match authenticate(&request.token, now) {
        None => return Ok(HttpResponse::Forbidden().finish()),
        Some(i) => i,
    };
    if account.owner != user_id {
        return Ok(HttpResponse::Forbidden().finish());
    }

    // input check
    match account.kind {
        AccountKind::TFSA | AccountKind::RRSP | AccountKind::FHSA => {
            if let Some(response) = rule_dep_wdl_cad(&request.transaction) {
                return Ok(response);
            }
        }
        _ => (),
    }

    request.transaction.insert()?;
    Ok(HttpResponse::Ok().finish())
}

fn rule_dep_wdl_cad(transaction: &Transaction) -> Option<HttpResponse> {
    match &transaction.action {
        TxnAction::Deposit(deposit) => {
            if deposit.value.1 != AssetId::CURRENCY(String::from("CAD")) {
                return Some(
                    HttpResponse::BadRequest()
                        .body("TFSA/RRSP/FHSA account can only deposit CAD"),
                );
            }
        }
        TxnAction::Withdrawal(withdrawal) => {
            if withdrawal.value.1 != AssetId::CURRENCY(String::from("CAD")) {
                return Some(
                    HttpResponse::BadRequest()
                        .body("TFSA/RRSP/FHSA account can only withdrawal CAD"),
                );
            }
        }
        _ => (),
    }

    None
}
