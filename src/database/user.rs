use crate::error::ServerError;
use core::str;
use rusqlite::{Connection, Row};
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
        connection: &mut Connection,
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

        let mut statement = connection.prepare(&query)?;
        let record: Option<Result<_, rusqlite::Error>> = statement
            .query_and_then(&*values.as_params(), |row| User::try_from(row))?
            .next();

        Ok(record.transpose()?)
    }

    pub fn by_username(
        username: impl Into<String>,
        connection: &mut Connection,
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

        let mut statement = connection.prepare(&query)?;
        let record: Option<Result<_, rusqlite::Error>> = statement
            .query_and_then(&*values.as_params(), |row| User::try_from(row))?
            .next();

        Ok(record.transpose()?)
    }

    pub fn delete(
        id: Uuid,
        connection: &mut Connection,
    ) -> Result<(), ServerError> {
        use super::account::AccountIden;
        use super::transaction::TransactionIden;

        // delete associated transactions
        let (query1, values1) = Query::delete()
            .from_table(TransactionIden::Table)
            .and_where(
                Expr::col(TransactionIden::Account).in_subquery(
                    Query::select()
                        .columns([AccountIden::Id])
                        .from(AccountIden::Table)
                        .and_where(Expr::col(AccountIden::Owner).eq(id))
                        .take(),
                ),
            )
            .build_rusqlite(SqliteQueryBuilder);

        // delete associated accounts
        let (query2, values2) = Query::delete()
            .from_table(AccountIden::Table)
            .and_where(Expr::col(AccountIden::Owner).eq(id))
            .build_rusqlite(SqliteQueryBuilder);

        // delete user
        let (query3, values3) = Query::delete()
            .from_table(UserIden::Table)
            .and_where(Expr::col(UserIden::Id).eq(id))
            .build_rusqlite(SqliteQueryBuilder);

        let transaction = connection.transaction()?;
        transaction.execute(&query1, &*values1.as_params())?;
        transaction.execute(&query2, &*values2.as_params())?;
        transaction.execute(&query3, &*values3.as_params())?;
        transaction.commit()?;
        Ok(())
    }

    pub fn insert(
        &self,
        connection: &mut Connection,
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

        connection.execute(&query, &*values.as_params())?;
        Ok(id)
    }

    pub fn update(
        &self,
        connection: &mut Connection,
    ) -> Result<(), ServerError> {
        let (query, values) = Query::update()
            .table(UserIden::Table)
            .values([
                (UserIden::Username, self.username.clone().into()),
                (UserIden::Password, self.password.clone().into()),
            ])
            .and_where(Expr::col(UserIden::Id).eq(self.id))
            .build_rusqlite(SqliteQueryBuilder);

        connection.execute(&query, &*values.as_params())?;
        Ok(())
    }

    pub fn attempts(
        &self,
        connection: &mut Connection,
    ) -> Result<u64, ServerError> {
        let (query, values) = Query::select()
            .expr_as(
                // if login_at is null, this condition will also fail
                Expr::case(
                    Expr::col(UserIden::LoginAt)
                        .gt(Expr::cust("DATETIME('NOW', '-1 minutes')")),
                    Expr::col(UserIden::Attempts),
                )
                .finally(0),
                UserIden::Attempts,
            )
            .from(UserIden::Table)
            .and_where(Expr::col(UserIden::Id).eq(self.id))
            .build_rusqlite(SqliteQueryBuilder);

        let mut statement = connection.prepare(&query)?;
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
        connection: &mut Connection,
    ) -> Result<(), ServerError> {
        let (query, values) = Query::update()
            .table(UserIden::Table)
            .values([
                (UserIden::LoginAt, Expr::cust("DATETIME('NOW')")),
                (
                    UserIden::Attempts,
                    // if login_at is null, this condition will also fail
                    Expr::case(
                        Expr::col(UserIden::LoginAt)
                            .gt(Expr::cust("DATETIME('NOW', '-1 minutes')")),
                        Expr::col(UserIden::Attempts).add(1),
                    )
                    .finally(1)
                    .into(),
                ),
            ])
            .and_where(Expr::col(UserIden::Id).eq(self.id))
            .build_rusqlite(SqliteQueryBuilder);

        connection.execute(&query, &*values.as_params())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database;
    use chrono::NaiveDate;
    use rust_decimal_macros::dec;
    use sha2::{Digest, Sha256};
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_insert_and_select() {
        let mut connection =
            Connection::open_in_memory().expect("fail to create database");
        database::migration::run_migration(&mut connection)
            .expect("database initialization fail");

        let username = "test_user";
        let mut u0 = User::new(username, Sha256::digest("password").to_vec());
        u0.id = u0.insert(&mut connection).expect("panic");
        assert_ne!(Uuid::nil(), u0.id);

        let u1 = User::by_username(username, &mut connection)
            .expect("panic")
            .expect("panic");
        assert_eq!(u0.id, u1.id);
        assert_eq!(u0.username, u1.username);
        assert_eq!(u0.password, u1.password);

        let u2 = User::by_id(u0.id, &mut connection)
            .expect("panic")
            .expect("panic");
        assert_eq!(u0.id, u2.id);
        assert_eq!(u0.username, u2.username);
        assert_eq!(u0.password, u2.password);
    }

    #[test]
    fn test_duplicate_insert() {
        let mut connection =
            Connection::open_in_memory().expect("fail to create database");
        database::migration::run_migration(&mut connection)
            .expect("database initialization fail");

        let username = "test_user";
        let mut u0 = User::new(username, Sha256::digest("password").to_vec());
        u0.id = u0.insert(&mut connection).expect("panic");
        let u1 = User::new(username, Sha256::digest("password").to_vec());
        u1.insert(&mut connection).expect_err("duplicate insert");
    }

    #[test]
    fn test_user_update() {
        let mut connection =
            Connection::open_in_memory().expect("fail to create database");
        database::migration::run_migration(&mut connection)
            .expect("database initialization fail");

        let mut u0 =
            User::new("test_user_0", Sha256::digest("password").to_vec());
        u0.id = u0.insert(&mut connection).expect("panic");

        u0.username = String::from("test_user_1");
        u0.update(&mut connection).expect("panic");
        let u1 = User::by_id(u0.id, &mut connection)
            .expect("panic")
            .expect("panic");
        assert_eq!(u0.username, u1.username);

        u0.password = Sha256::digest("some_random_password").to_vec();
        u0.update(&mut connection).expect("panic");
        let u1 = User::by_id(u0.id, &mut connection)
            .expect("panic")
            .expect("panic");
        assert_eq!(u0.password, u1.password);
    }

    #[test]
    fn test_delete() {
        use database::account::{Account, AccountKind};
        use database::asset::AssetId;
        use database::transaction::{Transaction, TxnAction};

        let mut connection =
            Connection::open_in_memory().expect("fail to create database");
        database::migration::run_migration(&mut connection)
            .expect("database initialization fail");

        let mut u0 =
            User::new("test_user", Sha256::digest("password").to_vec());
        u0.id = u0.insert(&mut connection).expect("panic");

        let mut a0 =
            Account::new("test_account", "alias", u0.id, AccountKind::NRA);
        a0.id = a0.insert(&mut connection).expect("panic");

        let mut t0 = Transaction::new(
            a0.id,
            NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            TxnAction::Deposit {
                value: (dec!(100.0), AssetId::currency("CAD")),
                fee: (dec!(0.0), AssetId::currency("CAD")),
            },
        );
        t0.id = t0.insert(&mut connection).expect("panic");

        // TODO: test user-attached asset deletion here

        User::delete(u0.id, &mut connection).expect("panic");
        assert_eq!(
            None,
            Transaction::by_id(t0.id, &mut connection).expect("panic")
        );
        assert_eq!(
            None,
            Account::by_id(a0.id, &mut connection).expect("panic")
        );
        assert_eq!(None, User::by_id(u0.id, &mut connection).expect("panic"));
    }

    #[test]
    fn test_attempts() {
        let mut connection =
            Connection::open_in_memory().expect("fail to create database");
        database::migration::run_migration(&mut connection)
            .expect("database initialization fail");

        let mut u0 =
            User::new("test_user", Sha256::digest("password").to_vec());
        u0.id = u0.insert(&mut connection).expect("panic");

        let a = u0.attempts(&mut connection).expect("panic");
        assert_eq!(0, a);

        u0.add_attempt(&mut connection).expect("panic");
        let a = u0.attempts(&mut connection).expect("panic");
        assert_eq!(1, a);

        u0.add_attempt(&mut connection).expect("panic");
        let a = u0.attempts(&mut connection).expect("panic");
        assert_eq!(2, a);

        u0.add_attempt(&mut connection).expect("panic");
        let a = u0.attempts(&mut connection).expect("panic");
        assert_eq!(3, a);

        thread::sleep(Duration::from_secs(70));

        let a = u0.attempts(&mut connection).expect("panic");
        assert_eq!(0, a);
    }
}
