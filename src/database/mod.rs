pub mod account;
pub mod user;
pub mod transaction;
pub mod asset;
mod migration;

use crate::error::ServerError;
use std::fs;

const DATABASE: &str = "data/sqlite.db";

pub fn init() -> Result<(), ServerError> {
    fs::create_dir_all("data/")?;
    migration::run_migration()?;
    Ok(())
}

pub use user::User;
pub use account::Account;
pub use transaction::Transaction;
