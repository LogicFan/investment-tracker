use crate::database::Account;
use crate::user;
use std::time::SystemTimeError;

pub mod delete;
pub mod fetch;
pub mod insert;
pub mod update;

pub fn authenticate(
    account: &Account,
    token: &String,
    _: &rusqlite::Transaction,
) -> Result<bool, SystemTimeError> {
    Ok(user::authenticate(&token)?
        .map(|user_id| account.owner == user_id)
        .unwrap_or(false))
}

pub fn validate(
    account: &Account,
    _: &rusqlite::Transaction,
) -> Option<&'static str> {
    if account.name.len() < 4 {
        Some("account name too short")
    } else if account.alias.len() < 4 {
        Some("account alias too short")
    } else {
        None
    }
}
