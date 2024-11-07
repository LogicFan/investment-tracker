mod action;

use crate::database::DATABASE;
use crate::error::ServerError;
pub use action::TxnAction;
use chrono::NaiveDate;
use rusqlite::{Connection, Row};
use sea_query::{enum_def, Expr, IdenStatic, Query, SqliteQueryBuilder};
use sea_query_rusqlite::RusqliteBinder;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[enum_def]
pub struct Transaction {
    #[serde(default)]
    pub id: Uuid,
    pub account: Uuid,
    pub date: NaiveDate,
    pub action: TxnAction,
}

impl PartialEq for Transaction {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Transaction {}

impl TryFrom<&Row<'_>> for Transaction {
    type Error = rusqlite::Error;

    fn try_from(value: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.get(TransactionIden::Id.as_str())?,
            account: value.get(TransactionIden::Account.as_str())?,
            date: value.get(TransactionIden::Date.as_str())?,
            action: value.get(TransactionIden::Action.as_str())?,
        })
    }
}

impl Transaction {
    pub fn account(&self) -> Option<super::Account> {
        match super::Account::by_id(self.account) {
            Ok(Some(account)) => Some(account),
            _ => None,
        }
    }
}

impl Transaction {
    pub fn by_id(id: Uuid) -> Result<Option<Transaction>, ServerError> {
        let (query, values) = Query::select()
            .columns([
                TransactionIden::Id,
                TransactionIden::Account,
                TransactionIden::Date,
                TransactionIden::Action,
            ])
            .from(TransactionIden::Table)
            .and_where(Expr::col(TransactionIden::Id).eq(id))
            .build_rusqlite(SqliteQueryBuilder);

        let connection = Connection::open(DATABASE)?;
        let mut statement = connection.prepare(&query)?;
        let record: Option<Result<_, rusqlite::Error>> = statement
            .query_and_then(&*values.as_params(), |row| {
                Transaction::try_from(row)
            })?
            .next();

        Ok(record.transpose()?)
    }

    pub fn by_account(account: Uuid) -> Result<Vec<Transaction>, ServerError> {
        let (query, values) = Query::select()
            .columns([
                TransactionIden::Id,
                TransactionIden::Account,
                TransactionIden::Date,
                TransactionIden::Action,
            ])
            .from(TransactionIden::Table)
            .and_where(Expr::col(TransactionIden::Account).eq(account))
            .build_rusqlite(SqliteQueryBuilder);

        let connection = Connection::open(DATABASE)?;
        let mut statement = connection.prepare(&query)?;
        let record: Result<Vec<_>, rusqlite::Error> = statement
            .query_and_then(&*values.as_params(), |row| {
                Transaction::try_from(row)
            })?
            .collect();

        Ok(record?)
    }

    pub fn insert(&self) -> Result<(), ServerError> {
        assert!(self.id.is_nil());

        let (query, values) = Query::insert()
            .into_table(TransactionIden::Table)
            .columns([
                TransactionIden::Id,
                TransactionIden::Account,
                TransactionIden::Date,
                TransactionIden::Action,
            ])
            .values([
                Uuid::new_v4().into(),
                self.account.into(),
                self.date.into(),
                self.action.clone().into(),
            ])?
            .build_rusqlite(SqliteQueryBuilder);

        let connection = Connection::open(DATABASE)?;
        connection.execute(&query, &*values.as_params())?;
        Ok(())
    }
}
