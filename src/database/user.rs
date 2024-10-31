use super::DATABASE;
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
        })
    }
}

impl User {
    pub fn insert(&self) -> Result<(), ServerError> {
        assert!(self.id.is_nil());

        let (query, values) = Query::insert()
            .into_table(UserIden::Table)
            .columns([UserIden::Id, UserIden::Username, UserIden::Password])
            .values([
                Uuid::new_v4().into(),
                self.username.clone().into(),
                self.password.clone().into(),
            ])?
            .build_rusqlite(SqliteQueryBuilder);

        let connection = Connection::open(DATABASE)?;
        connection.execute(&query, &*values.as_params())?;
        Ok(())
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

    pub fn delete(&self) -> Result<(), ServerError> {
        // TODO: remove transaction related to user.

        use super::account::AccountIden;
        let (query1, values1) = Query::delete()
            .from_table(AccountIden::Table)
            .and_where(Expr::col(AccountIden::Owner).eq(self.id))
            .build_rusqlite(SqliteQueryBuilder);

        let (query2, values2) = Query::delete()
            .from_table(UserIden::Table)
            .and_where(Expr::col(UserIden::Id).eq(self.id))
            .build_rusqlite(SqliteQueryBuilder);

        let mut connection = Connection::open(DATABASE)?;
        let transaction = connection.transaction()?;
        transaction.execute(&query1, &*values1.as_params())?;
        transaction.execute(&query2, &*values2.as_params())?;
        transaction.commit()?;
        Ok(())
    }

    pub fn select(id: Uuid) -> Result<Option<User>, ServerError> {
        let (query, values) = Query::select()
            .columns([UserIden::Id, UserIden::Username, UserIden::Password])
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

    pub fn select_by_username(
        username: String,
    ) -> Result<Option<User>, ServerError> {
        let (query, values) = Query::select()
            .columns([UserIden::Id, UserIden::Username, UserIden::Password])
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
}
