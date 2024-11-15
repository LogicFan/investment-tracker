pub mod delete;
pub mod fetch;
pub mod insert;
pub mod update;

use crate::database::account::AccountKind;
use crate::database::asset::AssetId;
use crate::database::transaction::TxnAction;
use crate::database::Transaction;
use crate::user::authenticate;
use std::time::SystemTimeError;

fn has_permission(
    transaction: &Transaction,
    token: &String,
) -> Result<bool, SystemTimeError> {
    Ok(authenticate(&token)?
        .map(|user_id| {
            transaction
                .account()
                .map(|account| account.owner == user_id)
                .unwrap_or(false)
        })
        .unwrap_or(false))
}

fn validate_input(transaction: &Transaction) -> Option<&'static str> {
    if let Some(account) = transaction.account() {
        match account.kind {
            AccountKind::TFSA | AccountKind::RRSP | AccountKind::FHSA if rule_dep_wdl_cad(transaction) => {
                Some("Canadian registered account can only deposit or withdrawal Canadian dollar")
            }
            _ => None
        }
    } else {
        Some("no account exists")
    }
}

fn rule_dep_wdl_cad(transaction: &Transaction) -> bool {
    match &transaction.action {
        TxnAction::Deposit { value, .. } => {
            if value.1 != AssetId::CURRENCY(String::from("CAD")) {
                return true;
            }
        }
        TxnAction::Withdrawal { value, .. } => {
            if value.1 != AssetId::CURRENCY(String::from("CAD")) {
                return true;
            }
        }
        _ => (),
    }

    false
}
