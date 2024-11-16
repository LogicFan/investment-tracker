pub mod account;
pub mod asset;
mod migration;
pub mod transaction;
pub mod user;

use crate::error::ServerError;
use std::fs;

pub fn get_connection() -> Result<Connection, rusqlite::Error> {
    const DATABASE: &str = "data/sqlite.db";
    Connection::open(DATABASE)
}

pub fn init() -> Result<(), ServerError> {
    fs::create_dir_all("data/")?;
    migration::run_migration(&mut get_connection()?)?;
    Ok(())
}

pub use account::Account;
use rusqlite::Connection;
pub use transaction::Transaction;
pub use user::User;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        init().expect("database initialization fail");
    }
}
