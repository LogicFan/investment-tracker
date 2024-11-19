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

    let mut conn = get_connection()?;
    let tran = conn.transaction()?;
    migration::run_migration(&tran)?;
    tran.commit()?;
    
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
    fn test_init() -> Result<(), ServerError> {
        let mut connection = Connection::open_in_memory()?;
        let transaction = connection.transaction()?;
        migration::run_migration(&transaction)?;
        transaction.commit()?;

        Ok(())
    }
}
