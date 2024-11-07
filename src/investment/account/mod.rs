use crate::database::Account;
use crate::user::authenticate;
use std::time::SystemTimeError;

pub mod delete;
pub mod fetch;
pub mod insert;
pub mod update;

fn has_permission(
    account: &Account,
    token: &String,
) -> Result<bool, SystemTimeError> {
    Ok(authenticate(&token)?
        .map(|user_id| account.owner == user_id)
        .unwrap_or(false))
}

fn validate_input(account: &Account) -> Option<&'static str> {
    if account.name.len() < 4 {
        return Some("account name too short");
    } else if account.alias.len() < 4 {
        return Some("account alias too short");
    } else {
        return None;
    }
}
