mod action;

use crate::error::ServerError;
pub use action::TxnAction;
use chrono::NaiveDate;
use rusqlite::Row;
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
    pub fn new(account: Uuid, date: NaiveDate, action: TxnAction) -> Self {
        Self {
            id: Uuid::nil(),
            account,
            date,
            action: action.clone(),
        }
    }

    pub fn account(
        &self,
        transaction: &rusqlite::Transaction,
    ) -> Option<super::Account> {
        match super::Account::by_id(self.account, &transaction) {
            Ok(Some(account)) => Some(account),
            _ => None,
        }
    }
}

impl Transaction {
    pub fn by_id(
        id: Uuid,
        transaction: &rusqlite::Transaction,
    ) -> Result<Option<Transaction>, ServerError> {
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

        let mut statement = transaction.prepare(&query)?;
        let record: Option<Result<_, rusqlite::Error>> = statement
            .query_and_then(&*values.as_params(), |row| {
                Transaction::try_from(row)
            })?
            .next();

        Ok(record.transpose()?)
    }

    pub fn by_account(
        account: Uuid,
        transaction: &rusqlite::Transaction,
    ) -> Result<Vec<Transaction>, ServerError> {
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

        let mut statement = transaction.prepare(&query)?;
        let record: Result<Vec<_>, rusqlite::Error> = statement
            .query_and_then(&*values.as_params(), |row| {
                Transaction::try_from(row)
            })?
            .collect();

        Ok(record?)
    }

    pub fn delete(
        id: Uuid,
        transaction: &rusqlite::Transaction,
    ) -> Result<(), ServerError> {
        let (query, values) = Query::delete()
            .from_table(TransactionIden::Table)
            .and_where(Expr::col(TransactionIden::Id).eq(id))
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
            .into_table(TransactionIden::Table)
            .columns([
                TransactionIden::Id,
                TransactionIden::Account,
                TransactionIden::Date,
                TransactionIden::Action,
            ])
            .values([
                id.into(),
                self.account.into(),
                self.date.into(),
                self.action.clone().into(),
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
            .table(TransactionIden::Table)
            .values([
                (TransactionIden::Account, self.account.into()),
                (TransactionIden::Date, self.date.into()),
                (TransactionIden::Action, self.action.clone().into()),
            ])
            .and_where(Expr::col(TransactionIden::Id).eq(self.id))
            .build_rusqlite(SqliteQueryBuilder);

        transaction.execute(&query, &*values.as_params())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::account::AccountKind;
    use crate::database::asset::AssetId;
    use crate::database::{self, Account, User};
    use rusqlite::Connection;
    use rust_decimal_macros::dec;
    use sha2::{Digest, Sha256};

    #[test]
    fn test_insert_and_select() -> Result<(), ServerError> {
        let mut conn = Connection::open_in_memory()?;

        let a0 = {
            let tran = conn.transaction()?;
            database::migration::run_migration(&tran)?;
            let mut u0 = User::new(
                String::from("test_user"),
                Sha256::digest("password").to_vec(),
            );
            u0.id = u0.insert(&tran)?;
            let mut a0 =
                Account::new("test_account", "alias", u0.id, AccountKind::NRA);
            a0.id = a0.insert(&tran)?;
            tran.commit()?;
            a0
        };
        let (t0, t1) = {
            let tran = conn.transaction()?;
            let mut t0 = Transaction::new(
                a0.id,
                NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
                TxnAction::Deposit {
                    value: (dec!(100.0), AssetId::currency("CAD")),
                    fee: (dec!(0.0), AssetId::currency("CAD")),
                },
            );
            t0.id = t0.insert(&tran)?;
            let mut t1 = Transaction::new(
                a0.id,
                NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
                TxnAction::Withdrawal {
                    value: (dec!(100.0), AssetId::currency("CAD")),
                    fee: (dec!(0.0), AssetId::currency("CAD")),
                },
            );
            t1.id = t1.insert(&tran)?;
            tran.commit()?;
            (t0, t1)
        };
        {
            let tran = conn.transaction()?;
            let res =
                Transaction::by_id(t0.id, &tran)?.expect("no transaction");
            assert_eq!(t0.id, res.id);
            assert_eq!(t0.date, res.date);
            assert_eq!(t0.action, res.action);
        }
        {
            let tran = conn.transaction()?;
            let res = Transaction::by_account(a0.id, &tran)?;
            assert!(res.contains(&t0));
            assert!(res.contains(&t1));
        }

        Ok(())
    }

    #[test]
    fn test_no_account() -> Result<(), ServerError> {
        let mut conn = Connection::open_in_memory()?;

        {
            let tran = conn.transaction()?;
            database::migration::run_migration(&tran)?;
            tran.commit()?;
        }
        {
            let tran = conn.transaction()?;
            let t0 = Transaction::new(
                Uuid::nil(),
                NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
                TxnAction::Withdrawal {
                    value: (dec!(100.0), AssetId::currency("CAD")),
                    fee: (dec!(0.0), AssetId::currency("CAD")),
                },
            );
            t0.insert(&tran)
                .expect_err("insert transaction with invalid account");
        }

        Ok(())
    }

    #[test]
    fn test_update() -> Result<(), ServerError> {
        let mut conn = Connection::open_in_memory()?;

        let mut t0 = {
            let tran = conn.transaction()?;
            database::migration::run_migration(&tran)?;
            let mut u0 = User::new(
                String::from("test_user"),
                Sha256::digest("password").to_vec(),
            );
            u0.id = u0.insert(&tran)?;
            let mut a0 =
                Account::new("test_account", "alias", u0.id, AccountKind::NRA);
            a0.id = a0.insert(&tran)?;
            let mut t0 = Transaction::new(
                a0.id,
                NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
                TxnAction::Deposit {
                    value: (dec!(100.0), AssetId::currency("CAD")),
                    fee: (dec!(0.0), AssetId::currency("CAD")),
                },
            );
            t0.id = t0.insert(&tran)?;
            tran.commit()?;
            t0
        };
        {
            let tran = conn.transaction()?;
            t0.date = NaiveDate::from_ymd_opt(2021, 1, 1).unwrap();
            t0.update(&tran)?;
            tran.commit()?;
            let tran = conn.transaction()?;
            let res =
                Transaction::by_id(t0.id, &tran)?.expect("no transaction");
            assert_eq!(t0.date, res.date);
        }
        {
            let tran = conn.transaction()?;
            t0.action = TxnAction::Fee {
                value: (dec!(1.0), AssetId::currency("CAD")),
                reason: String::from("Management Fee"),
            };
            t0.update(&tran)?;
            tran.commit()?;
            let tran = conn.transaction()?;
            let res =
                Transaction::by_id(t0.id, &tran)?.expect("no transaction");
            assert_eq!(t0.date, res.date);
        }
        Ok(())
    }

    #[test]
    fn test_delete() -> Result<(), ServerError> {
        let mut conn = Connection::open_in_memory()?;

        let t0 = {
            let tran = conn.transaction()?;
            database::migration::run_migration(&tran)?;
            let mut u0 = User::new(
                String::from("test_user"),
                Sha256::digest("password").to_vec(),
            );
            u0.id = u0.insert(&tran)?;
            let mut a0 =
                Account::new("test_account", "alias", u0.id, AccountKind::NRA);
            a0.id = a0.insert(&tran)?;
            let mut t0 = Transaction::new(
                a0.id,
                NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
                TxnAction::Deposit {
                    value: (dec!(100.0), AssetId::currency("CAD")),
                    fee: (dec!(0.0), AssetId::currency("CAD")),
                },
            );
            t0.id = t0.insert(&tran)?;
            tran.commit()?;
            t0
        };
        {
            let tran = conn.transaction()?;
            Transaction::delete(t0.id, &tran)?;
            assert_eq!(None, Transaction::by_id(t0.id, &tran)?);
        }
        Ok(())
    }
}
