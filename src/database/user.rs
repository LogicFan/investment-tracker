use crate::error::ServerError;
use core::str;
use rusqlite::Row;
use sea_query::{enum_def, Expr, IdenStatic, Query, SqliteQueryBuilder};
use sea_query_rusqlite::RusqliteBinder;
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[enum_def]
pub struct User {
    #[serde(default)]
    pub id: Uuid,
    pub username: String,
    pub password: Vec<u8>,
    #[serde(default)]
    pub login_at: (),
    #[serde(default)]
    pub attempts: (),
}

impl PartialEq for User {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for User {}

impl TryFrom<&Row<'_>> for User {
    type Error = rusqlite::Error;

    fn try_from(value: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.get(UserIden::Id.as_str())?,
            username: value.get(UserIden::Username.as_str())?,
            password: value.get(UserIden::Password.as_str())?,
            login_at: (),
            attempts: (),
        })
    }
}

impl User {
    pub fn new(
        username: impl Into<String>,
        password: impl Into<Vec<u8>>,
    ) -> Self {
        Self {
            id: Uuid::nil(),
            username: username.into(),
            password: password.into(),
            login_at: (),
            attempts: (),
        }
    }
}

impl User {
    pub fn by_id(
        id: Uuid,
        transaction: &rusqlite::Transaction,
    ) -> Result<Option<User>, ServerError> {
        let (query, values) = Query::select()
            .columns([
                UserIden::Id,
                UserIden::Username,
                UserIden::Password,
                UserIden::LoginAt,
                UserIden::Attempts,
            ])
            .from(UserIden::Table)
            .and_where(Expr::col(UserIden::Id).eq(id))
            .build_rusqlite(SqliteQueryBuilder);

        let mut statement = transaction.prepare(&query)?;
        let record = statement
            .query_and_then(&*values.as_params(), |row| User::try_from(row))?
            .next();

        Ok(record.transpose()?)
    }

    pub fn by_username(
        username: impl Into<String>,
        transaction: &rusqlite::Transaction,
    ) -> Result<Option<User>, ServerError> {
        let (query, values) = Query::select()
            .columns([
                UserIden::Id,
                UserIden::Username,
                UserIden::Password,
                UserIden::LoginAt,
                UserIden::Attempts,
            ])
            .from(UserIden::Table)
            .and_where(Expr::col(UserIden::Username).eq(username.into()))
            .build_rusqlite(SqliteQueryBuilder);

        let mut statement = transaction.prepare(&query)?;
        let record = statement
            .query_and_then(&*values.as_params(), |row| User::try_from(row))?
            .next();

        Ok(record.transpose()?)
    }

    pub fn delete(
        id: Uuid,
        transaction: &rusqlite::Transaction,
    ) -> Result<(), ServerError> {
        {
            use super::account::{Account, AccountIden};
            let (query, values) = Query::select()
                .columns([AccountIden::Id])
                .from(AccountIden::Table)
                .and_where(Expr::col(AccountIden::Owner).eq(id))
                .build_rusqlite(SqliteQueryBuilder);
            let mut statement = transaction.prepare(&query)?;
            statement
                .query_and_then(&*values.as_params(), |row| row.get(0))?
                .try_for_each(|x: Result<Uuid, _>| {
                    Account::delete(x?, &transaction)
                })?;
        }

        // delete user
        let (query, values) = Query::delete()
            .from_table(UserIden::Table)
            .and_where(Expr::col(UserIden::Id).eq(id))
            .build_rusqlite(SqliteQueryBuilder);

        transaction.execute(&query, &*values.as_params())?;
        Ok(())
    }

    pub fn insert(
        &self,
        transaction: &rusqlite::Transaction,
    ) -> Result<Uuid, ServerError> {
        assert!(self.id.is_nil());

        let id = Uuid::new_v4();
        let (query, values) = Query::insert()
            .into_table(UserIden::Table)
            .columns([UserIden::Id, UserIden::Username, UserIden::Password])
            .values([
                id.into(),
                self.username.clone().into(),
                self.password.clone().into(),
            ])?
            .build_rusqlite(SqliteQueryBuilder);

        transaction.execute(&query, &*values.as_params())?;
        Ok(id)
    }

    pub fn update(
        &self,
        transaction: &rusqlite::Transaction,
    ) -> Result<(), ServerError> {
        let (query, values) = Query::update()
            .table(UserIden::Table)
            .values([
                (UserIden::Username, self.username.clone().into()),
                (UserIden::Password, self.password.clone().into()),
            ])
            .and_where(Expr::col(UserIden::Id).eq(self.id))
            .build_rusqlite(SqliteQueryBuilder);

        transaction.execute(&query, &*values.as_params())?;
        Ok(())
    }

    #[cfg(not(test))]
    const ATTEMPT_TIMEOUT: &'static str = "-1 minutes";
    #[cfg(test)]
    const ATTEMPT_TIMEOUT: &'static str = "-1 seconds";

    pub fn attempts(
        &self,
        transaction: &rusqlite::Transaction,
    ) -> Result<u64, ServerError> {
        let (query, values) = Query::select()
            .expr_as(
                // if login_at is null, this condition will also fail
                Expr::case(
                    Expr::col(UserIden::LoginAt).gt(Expr::cust(format!(
                        "DATETIME('NOW', '{}')",
                        Self::ATTEMPT_TIMEOUT
                    ))),
                    Expr::col(UserIden::Attempts),
                )
                .finally(0),
                UserIden::Attempts,
            )
            .from(UserIden::Table)
            .and_where(Expr::col(UserIden::Id).eq(self.id))
            .build_rusqlite(SqliteQueryBuilder);

        let mut statement = transaction.prepare(&query)?;
        let record: Result<u64, rusqlite::Error> = statement
            .query_and_then(&*values.as_params(), |row| {
                row.get(UserIden::Attempts.as_str())
            })?
            .next()
            .unwrap_or(Ok(0));

        Ok(record?)
    }

    pub fn add_attempt(
        &self,
        transaction: &rusqlite::Transaction,
    ) -> Result<(), ServerError> {
        let (query, values) = Query::update()
            .table(UserIden::Table)
            .values([
                (UserIden::LoginAt, Expr::cust("DATETIME('NOW')")),
                (
                    UserIden::Attempts,
                    // if login_at is null, this condition will also fail
                    Expr::case(
                        Expr::col(UserIden::LoginAt).gt(Expr::cust(format!(
                            "DATETIME('NOW', '{}')",
                            Self::ATTEMPT_TIMEOUT
                        ))),
                        Expr::col(UserIden::Attempts).add(1),
                    )
                    .finally(1)
                    .into(),
                ),
            ])
            .and_where(Expr::col(UserIden::Id).eq(self.id))
            .build_rusqlite(SqliteQueryBuilder);

        transaction.execute(&query, &*values.as_params())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database;
    use chrono::NaiveDate;
    use rusqlite::Connection;
    use rust_decimal_macros::dec;
    use sha2::{Digest, Sha256};
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_insert_and_select() -> Result<(), ServerError> {
        let mut conn = Connection::open_in_memory()?;
        let username = "test_user";

        {
            let tran = conn.transaction()?;
            database::migration::run_migration(&tran)?;
            tran.commit()?;
        }
        let u0 = {
            let tran = conn.transaction()?;
            let mut u0 =
                User::new(username, Sha256::digest("password").to_vec());
            u0.id = u0.insert(&tran)?;
            assert_ne!(Uuid::nil(), u0.id);
            tran.commit()?;
            u0
        };
        {
            let tran = conn.transaction()?;
            let res = User::by_username(username, &tran)?.expect("no user");
            assert_eq!(u0.id, res.id);
            assert_eq!(u0.username, res.username);
            assert_eq!(u0.password, res.password);
        }
        {
            let tran = conn.transaction()?;
            let res = User::by_id(u0.id, &tran)?.expect("no user");
            assert_eq!(u0.id, res.id);
            assert_eq!(u0.username, res.username);
            assert_eq!(u0.password, res.password);
        }

        Ok(())
    }

    #[test]
    fn test_duplicate_insert() -> Result<(), ServerError> {
        let mut conn = Connection::open_in_memory()?;
        let username = "test_user";

        {
            let tran = conn.transaction()?;
            database::migration::run_migration(&tran)?;
            tran.commit()?;
        }
        {
            let tran = conn.transaction()?;
            let mut u0 =
                User::new(username, Sha256::digest("password").to_vec());
            u0.id = u0.insert(&tran)?;
            tran.commit()?;
        }
        {
            let tran = conn.transaction()?;
            let u1 = User::new(username, Sha256::digest("password").to_vec());
            u1.insert(&tran).expect_err("duplicate insert");
        }

        Ok(())
    }

    #[test]
    fn test_user_update() -> Result<(), ServerError> {
        let mut conn = Connection::open_in_memory()?;

        {
            let tran = conn.transaction()?;
            database::migration::run_migration(&tran)?;
            tran.commit()?;
        }
        let mut u0 = {
            let tran = conn.transaction()?;
            let mut u0 =
                User::new("test_user_0", Sha256::digest("password").to_vec());
            u0.id = u0.insert(&tran)?;
            tran.commit()?;
            u0
        };
        {
            let tran = conn.transaction()?;
            u0.username = String::from("test_user_1");
            u0.update(&tran)?;
            tran.commit()?;

            let tran = conn.transaction()?;
            let res = User::by_id(u0.id, &tran)?.expect("no user");
            assert_eq!(u0.username, res.username);
        }
        {
            let tran = conn.transaction()?;
            u0.password = Sha256::digest("some_random_password").to_vec();
            u0.update(&tran)?;
            tran.commit()?;

            let tran = conn.transaction()?;
            let res = User::by_id(u0.id, &tran)?.expect("no user");
            assert_eq!(u0.password, res.password);
        }
        Ok(())
    }

    #[test]
    fn test_delete() -> Result<(), ServerError> {
        use database::account::{Account, AccountKind};
        use database::asset::AssetId;
        use database::transaction::{Transaction, TxnAction};
        let mut conn = Connection::open_in_memory()?;

        {
            let tran = conn.transaction()?;
            database::migration::run_migration(&tran)?;
            tran.commit()?;
        }
        let (u0, a0, t0) = {
            let tran = conn.transaction()?;
            let mut u0 =
                User::new("test_user", Sha256::digest("password").to_vec());
            u0.id = u0.insert(&tran)?;
            let mut a0 =
                Account::new("test_account", "alias", u0.id, AccountKind::NRA);
            a0.id = a0.insert(&tran)?;
            tran.commit()?; // TODO: move to the end once all other part support transaction level operation.
            let mut t0 = Transaction::new(
                a0.id,
                NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
                TxnAction::Deposit {
                    value: (dec!(100.0), AssetId::currency("CAD")),
                    fee: (dec!(0.0), AssetId::currency("CAD")),
                },
            );
            t0.id = t0.insert(&mut conn)?;
            // TODO: test user-attached asset deletion here
            (u0, a0, t0)
        };
        {
            let tran = conn.transaction()?;
            User::delete(u0.id, &tran)?;
            tran.commit()?;

            let tran = conn.transaction()?;
            // assert_eq!(None, Transaction::by_id(t0.id, &mut conn)?);
            assert_eq!(None, Account::by_id(a0.id, &tran)?);
            assert_eq!(None, User::by_id(u0.id, &tran)?);
            tran.rollback()?; // TODO: remove

            assert_eq!(None, Transaction::by_id(t0.id, &mut conn)?);
        }

        Ok(())
    }

    #[test]
    fn test_attempts() -> Result<(), ServerError> {
        let mut conn = Connection::open_in_memory()?;

        {
            let tran = conn.transaction()?;
            database::migration::run_migration(&tran)?;
            tran.commit()?;
        }
        let u0 = {
            let tran = conn.transaction()?;
            let mut u0 =
                User::new("test_user", Sha256::digest("password").to_vec());
            u0.id = u0.insert(&tran)?;
            tran.commit()?;
            u0
        };
        {
            let tran = conn.transaction()?;
            let a = u0.attempts(&tran)?;
            assert_eq!(0, a);
        }
        {
            let tran = conn.transaction()?;
            u0.add_attempt(&tran)?;
            tran.commit()?;

            let tran = conn.transaction()?;
            let res = u0.attempts(&tran)?;
            assert_eq!(1, res);
        }
        {
            let tran = conn.transaction()?;
            u0.add_attempt(&tran)?;
            tran.commit()?;

            let tran = conn.transaction()?;
            let res = u0.attempts(&tran)?;
            assert_eq!(2, res);
        }
        {
            let tran = conn.transaction()?;
            u0.add_attempt(&tran)?;
            tran.commit()?;

            let tran = conn.transaction()?;
            let res = u0.attempts(&tran)?;
            assert_eq!(3, res);
        }

        {
            thread::sleep(Duration::from_secs(2));
            let tran = conn.transaction()?;
            let res = u0.attempts(&tran)?;
            assert_eq!(0, res);
        }

        Ok(())
    }
}
