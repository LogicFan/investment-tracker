use super::DATABASE;
use crate::error::ServerError;
use chrono::{DateTime, Duration, Utc};
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
    pub login_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub attempts: u64,
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
            login_at: value.get(UserIden::LoginAt.as_str())?,
            attempts: value.get(UserIden::Attempts.as_str())?,
        })
    }
}

impl User {
    pub fn by_id(id: Uuid) -> Result<Option<User>, ServerError> {
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

        let connection = Connection::open(DATABASE)?;
        let mut statement = connection.prepare(&query)?;
        let record: Option<Result<_, rusqlite::Error>> = statement
            .query_and_then(&*values.as_params(), |row| User::try_from(row))?
            .next();

        Ok(record.transpose()?)
    }

    pub fn by_username(username: String) -> Result<Option<User>, ServerError> {
        let (query, values) = Query::select()
            .columns([
                UserIden::Id,
                UserIden::Username,
                UserIden::Password,
                UserIden::LoginAt,
                UserIden::Attempts,
            ])
            .from(UserIden::Table)
            .and_where(Expr::col(UserIden::Username).eq(username))
            .build_rusqlite(SqliteQueryBuilder);

        let connection = Connection::open(DATABASE)?;
        let mut statement = connection.prepare(&query)?;
        let record: Option<Result<_, rusqlite::Error>> = statement
            .query_and_then(&*values.as_params(), |row| User::try_from(row))?
            .next();

        Ok(record.transpose()?)
    }

    pub fn delete(id: Uuid) -> Result<(), ServerError> {
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

        let mut connection = Connection::open(DATABASE)?;
        let transaction = connection.transaction()?;
        transaction.execute(&query1, &*values1.as_params())?;
        transaction.execute(&query2, &*values2.as_params())?;
        transaction.execute(&query3, &*values3.as_params())?;
        transaction.commit()?;
        Ok(())
    }

    pub fn insert(&self) -> Result<Uuid, ServerError> {
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

        let connection = Connection::open(DATABASE)?;
        connection.execute(&query, &*values.as_params())?;
        Ok(id)
    }

    pub fn update(&self) -> Result<(), ServerError> {
        let (query, values) = Query::update()
            .table(UserIden::Table)
            .values([
                (UserIden::Username, self.username.clone().into()),
                (UserIden::Password, self.password.clone().into()),
            ])
            .and_where(Expr::col(UserIden::Id).eq(self.id))
            .build_rusqlite(SqliteQueryBuilder);

        let connection = Connection::open(DATABASE)?;
        connection.execute(&query, &*values.as_params())?;
        Ok(())
    }

    pub fn attempts(&self, ttl: Duration) -> u64 {
        if let Some(login_at) = self.login_at {
            if login_at + ttl > Utc::now() {
                return self.attempts;
            }
        }
        return 0;
    }

    pub fn add_attempt(&self, ttl: Duration) -> Result<(), ServerError> {
        let attempts = self.attempts(ttl) + 1;

        let (query, values) = Query::update()
            .table(UserIden::Table)
            .values([
                (UserIden::LoginAt, Expr::cust("DATETIME('NOW')")),
                (UserIden::Attempts, attempts.into()),
            ])
            .and_where(Expr::col(UserIden::Id).eq(self.id))
            .build_rusqlite(SqliteQueryBuilder);

        let connection = Connection::open(DATABASE)?;
        connection.execute(&query, &*values.as_params())?;

        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database;
    use sha2::{Digest, Sha256};

    #[test]
    fn test_insert_and_select() {
        database::init().expect("database initialization fail");

        let username = String::from("test_insert_and_select");
        let u0 = User {
            id: Uuid::nil(),
            username: username.clone(),
            password: Sha256::digest("test_password_1").to_vec(),
            login_at: None,
            attempts: 0,
        };
        let id = u0.insert().expect("insert panic");
        assert_ne!(Uuid::nil(), id);

        let u1 = User::by_username(username.clone())
            .expect("select by username panic")
            .expect("user does not exist");
        assert_eq!(id, u1.id);
        assert_eq!(u0.username, u1.username);
        assert_eq!(u0.password, u1.password);

        let u2 = User::by_id(id)
            .expect("select by id panic")
            .expect("user does not exist");
        assert_eq!(id, u2.id);
        assert_eq!(u0.username, u2.username);
        assert_eq!(u0.password, u2.password);

        // clean up
        User::delete(id).expect("test clean-up fail");
    }

    #[test]
    fn test_duplicate_insert() {
        database::init().expect("database initialization fail");

        let username = String::from("test_duplicate_insert");
        let u0 = User {
            id: Uuid::nil(),
            username: username.clone(),
            password: Sha256::digest("test_password_2").to_vec(),
            login_at: None,
            attempts: 0,
        };
        let id = u0.insert().expect("insert panic");
        let u1 = User {
            id: Uuid::nil(),
            username: username.clone(),
            password: Sha256::digest("test_password_3").to_vec(),
            login_at: None,
            attempts: 0,
        };
        u1.insert().expect_err("duplicate insert");

        // clean up
        User::delete(id).expect("test clean-up fail");
    }

    #[test]
    fn test_update() {
        database::init().expect("database initialization fail");

        let mut u0 = User {
            id: Uuid::nil(),
            username: String::from("test_update"),
            password: Sha256::digest("test_password_4").to_vec(),
            login_at: None,
            attempts: 0,
        };
        let id = u0.insert().expect("insert panic");
        u0.id = id;

        u0.username = String::from("test_update_2");
        u0.update().expect("update fail");
        let u1 = User::by_id(id)
            .expect("select panic")
            .expect("user does not exist");
        assert_eq!(u0.username, u1.username);

        u0.password = Sha256::digest("some_random_password").to_vec();
        u0.update().expect("update fail");
        let u1 = User::by_id(id)
            .expect("select panic")
            .expect("user does not exist");
        assert_eq!(u0.password, u1.password);

        // should not be updated
        u0.login_at = Some(Utc::now());
        u0.update().expect("update fail");
        let u1 = User::by_id(id)
            .expect("select panic")
            .expect("user does not exist");
        assert_eq!(None, u1.login_at);

        // should not be updated
        u0.attempts = 10;
        u0.update().expect("update fail");
        let u1 = User::by_id(id)
            .expect("select panic")
            .expect("user does not exist");
        assert_eq!(0, u1.attempts);

        // clean up
        User::delete(id).expect("test clean-up fail");
    }
}
