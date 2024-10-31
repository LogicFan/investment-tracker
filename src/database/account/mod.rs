mod kind;

use super::DATABASE;
use crate::error::ServerError;
use core::str;
pub use kind::AccountKind;
use rusqlite::Connection;
use sea_query::{enum_def, Expr, Query, SqliteQueryBuilder};
use sea_query_rusqlite::RusqliteBinder;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[enum_def]
pub struct Account {
    #[serde(default)]
    pub id: Uuid,
    pub name: String,
    pub alias: String,
    pub owner: Uuid,
    pub kind: AccountKind,
}

impl PartialEq for Account {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Account {}

impl Account {
    pub fn resolve_owner(&self) -> Option<super::user::User> {
        match super::User::by_id(self.owner) {
            Ok(Some(user)) => Some(user),
            _ => None,
        }
    }
}

pub fn insert(account: Account) -> Result<(), ServerError> {
    assert!(account.id.is_nil());

    let (query, values) = Query::insert()
        .into_table(AccountIden::Table)
        .columns([
            AccountIden::Id,
            AccountIden::Name,
            AccountIden::Alias,
            AccountIden::Owner,
            AccountIden::Kind,
        ])
        .values([
            Uuid::new_v4().into(),
            account.name.into(),
            account.alias.into(),
            account.owner.into(),
            account.kind.into(),
        ])?
        .build_rusqlite(SqliteQueryBuilder);

    let connection = Connection::open(DATABASE)?;
    connection.execute(&query, &*values.as_params())?;
    Ok(())
}

pub fn update(account: Account) -> Result<(), ServerError> {
    let (query, values) = Query::update()
        .table(AccountIden::Table)
        .values([
            (AccountIden::Name, account.name.into()),
            (AccountIden::Alias, account.alias.into()),
            (AccountIden::Owner, account.owner.into()),
            (AccountIden::Kind, account.kind.into()),
        ])
        .and_where(Expr::col(AccountIden::Id).eq(account.id))
        .build_rusqlite(SqliteQueryBuilder);

    let connection = Connection::open(DATABASE)?;
    connection.execute(&query, &*values.as_params())?;
    Ok(())
}

pub fn delete(id: Uuid) -> Result<(), ServerError> {
    // TODO: also delete transaction related to this account.

    let (query, values) = Query::delete()
        .from_table(AccountIden::Table)
        .and_where(Expr::col(AccountIden::Id).eq(id))
        .build_rusqlite(SqliteQueryBuilder);

    let connection = Connection::open(DATABASE)?;
    connection.execute(&query, &*values.as_params())?;
    Ok(())
}

pub fn select(id: Uuid) -> Result<Option<Account>, ServerError> {
    let (query, values) = Query::select()
        .columns([
            AccountIden::Id,
            AccountIden::Name,
            AccountIden::Alias,
            AccountIden::Owner,
            AccountIden::Kind,
        ])
        .from(AccountIden::Table)
        .and_where(Expr::col(AccountIden::Id).eq(id))
        .build_rusqlite(SqliteQueryBuilder);

    let connection = Connection::open(DATABASE)?;
    let mut statement = connection.prepare(&query)?;
    let record: Option<Result<_, rusqlite::Error>> = statement
        .query_and_then(&*values.as_params(), |row| {
            Ok(Account {
                id: row.get(0)?,
                name: row.get(1)?,
                alias: row.get(2)?,
                owner: row.get(3)?,
                kind: row.get(4)?,
            })
        })?
        .next();

    Ok(record.transpose()?)
}

pub fn select_by_user(user_id: Uuid) -> Result<Vec<Account>, ServerError> {
    let (query, values) = Query::select()
        .columns([
            AccountIden::Id,
            AccountIden::Name,
            AccountIden::Alias,
            AccountIden::Owner,
            AccountIden::Kind,
        ])
        .from(AccountIden::Table)
        .and_where(Expr::col(AccountIden::Owner).eq(user_id))
        .build_rusqlite(SqliteQueryBuilder);

    let connection = Connection::open(DATABASE)?;
    let mut statement = connection.prepare(&query)?;
    let record: Result<Vec<_>, rusqlite::Error> = statement
        .query_and_then(&*values.as_params(), |row| {
            Ok(Account {
                id: row.get(0)?,
                name: row.get(1)?,
                alias: row.get(2)?,
                owner: row.get(3)?,
                kind: row.get(4)?,
            })
        })?
        .collect();

    Ok(record?)
}
