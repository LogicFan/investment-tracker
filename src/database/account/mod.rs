mod kind;

use crate::error::ServerError;
use core::str;
pub use kind::AccountKind;
use rusqlite::{Row, Transaction as SqlTransaction};
use sea_query::{enum_def, Expr, IdenStatic, Query, SqliteQueryBuilder};
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

impl TryFrom<&Row<'_>> for Account {
    type Error = rusqlite::Error;

    fn try_from(value: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.get(AccountIden::Id.as_str())?,
            name: value.get(AccountIden::Name.as_str())?,
            alias: value.get(AccountIden::Alias.as_str())?,
            owner: value.get(AccountIden::Owner.as_str())?,
            kind: value.get(AccountIden::Kind.as_str())?,
        })
    }
}

impl Account {
    pub fn new(
        name: impl Into<String>,
        alias: impl Into<String>,
        owner: Uuid,
        kind: AccountKind,
    ) -> Self {
        Self {
            id: Uuid::nil(),
            name: name.into(),
            alias: alias.into(),
            owner,
            kind,
        }
    }

    pub fn owner(&self, transaction: &SqlTransaction) -> Option<super::User> {
        match super::User::by_id(self.owner, &transaction) {
            Ok(Some(user)) => Some(user),
            _ => None,
        }
    }
}

impl Account {
    pub fn by_id(
        id: Uuid,
        transaction: &SqlTransaction,
    ) -> Result<Option<Account>, ServerError> {
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

        let mut statement = transaction.prepare(&query)?;
        let record: Option<Result<_, rusqlite::Error>> = statement
            .query_and_then(&*values.as_params(), |row| Account::try_from(row))?
            .next();

        Ok(record.transpose()?)
    }

    pub fn by_owner(
        owner: Uuid,
        transaction: &SqlTransaction,
    ) -> Result<Vec<Account>, ServerError> {
        let (query, values) = Query::select()
            .columns([
                AccountIden::Id,
                AccountIden::Name,
                AccountIden::Alias,
                AccountIden::Owner,
                AccountIden::Kind,
            ])
            .from(AccountIden::Table)
            .and_where(Expr::col(AccountIden::Owner).eq(owner))
            .build_rusqlite(SqliteQueryBuilder);

        let mut statement = transaction.prepare(&query)?;
        let record: Result<Vec<_>, rusqlite::Error> = statement
            .query_and_then(&*values.as_params(), |row| Account::try_from(row))?
            .collect();

        Ok(record?)
    }

    pub fn delete(
        id: Uuid,
        transaction: &SqlTransaction,
    ) -> Result<(), ServerError> {
        {
            use super::transaction::{Transaction, TransactionIden};
            let (query, values) = Query::select()
                .columns([TransactionIden::Id])
                .from(TransactionIden::Table)
                .and_where(Expr::col(TransactionIden::Account).eq(id))
                .build_rusqlite(SqliteQueryBuilder);
            let mut statement = transaction.prepare(&query)?;
            statement
                .query_and_then(&*values.as_params(), |row| row.get(0))?
                .try_for_each(|x: Result<Uuid, _>| {
                    Transaction::delete(x?, &transaction)
                })?;
        }

        let (query, values) = Query::delete()
            .from_table(AccountIden::Table)
            .and_where(Expr::col(AccountIden::Id).eq(id))
            .build_rusqlite(SqliteQueryBuilder);

        transaction.execute(&query, &*values.as_params())?;
        Ok(())
    }

    pub fn insert(
        &self,
        transaction: &SqlTransaction,
    ) -> Result<Uuid, ServerError> {
        assert!(self.id.is_nil());

        let id = Uuid::new_v4();
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
                id.into(),
                self.name.clone().into(),
                self.alias.clone().into(),
                self.owner.into(),
                self.kind.into(),
            ])?
            .build_rusqlite(SqliteQueryBuilder);

        transaction.execute(&query, &*values.as_params())?;
        Ok(id)
    }

    pub fn update(
        &self,
        transaction: &SqlTransaction,
    ) -> Result<(), ServerError> {
        let (query, values) = Query::update()
            .table(AccountIden::Table)
            .values([
                (AccountIden::Name, self.name.clone().into()),
                (AccountIden::Alias, self.alias.clone().into()),
                (AccountIden::Owner, self.owner.into()),
                (AccountIden::Kind, self.kind.into()),
            ])
            .and_where(Expr::col(AccountIden::Id).eq(self.id))
            .build_rusqlite(SqliteQueryBuilder);

        transaction.execute(&query, &*values.as_params())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::{self, User};
    use chrono::NaiveDate;
    use rusqlite::Connection;
    use rust_decimal_macros::dec;
    use sha2::{Digest, Sha256};

    #[test]
    fn test_insert_and_select() -> Result<(), ServerError> {
        let mut conn = Connection::open_in_memory()?;

        let u0 = {
            let tran = conn.transaction()?;
            database::migration::run_migration(&tran)?;
            let mut u0 = User::new(
                String::from("test_user"),
                Sha256::digest("password").to_vec(),
            );
            u0.id = u0.insert(&tran)?;
            tran.commit()?;
            u0
        };
        let (a0, a1) = {
            let tran = conn.transaction()?;
            let mut a0 = Account::new(
                "test_account_0",
                "alias",
                u0.id,
                AccountKind::NRA,
            );
            a0.id = a0.insert(&tran)?;
            let mut a1 = Account::new(
                "test_account_1",
                "alias2",
                u0.id,
                AccountKind::TFSA,
            );
            a1.id = a1.insert(&tran)?;
            tran.commit()?;
            (a0, a1)
        };
        {
            let tran = conn.transaction()?;
            let res = Account::by_id(a0.id, &tran)?.expect("no account");
            assert_eq!(a0.id, res.id);
            assert_eq!(a0.name, res.name);
            assert_eq!(a0.alias, res.alias);
            assert_eq!(a0.owner, res.owner);
            assert_eq!(a0.kind, res.kind);
        }
        {
            let tran = conn.transaction()?;
            let res = Account::by_owner(u0.id, &tran)?;
            assert!(res.contains(&a0));
            assert!(res.contains(&a1));
        }

        Ok(())
    }

    #[test]
    fn test_no_owner() -> Result<(), ServerError> {
        let mut conn = Connection::open_in_memory()?;

        {
            let tran = conn.transaction()?;
            database::migration::run_migration(&tran)?;
            tran.commit()?;
        }
        {
            let tran = conn.transaction()?;
            let a0 = Account::new(
                "test_account",
                "alias",
                Uuid::nil(),
                AccountKind::NRA,
            );
            a0.insert(&tran)
                .expect_err("insert account with invalid owner");
        }

        Ok(())
    }

    #[test]
    fn test_update() -> Result<(), ServerError> {
        let mut conn = Connection::open_in_memory()?;

        let u0 = {
            let tran = conn.transaction()?;
            database::migration::run_migration(&tran)?;
            let mut u0 = User::new(
                String::from("test_user"),
                Sha256::digest("password").to_vec(),
            );
            u0.id = u0.insert(&tran)?;
            tran.commit()?;
            u0
        };
        let mut a0 = {
            let tran = conn.transaction()?;
            let mut a0 = Account::new(
                "test_account_0",
                "alias",
                u0.id,
                AccountKind::NRA,
            );
            a0.id = a0.insert(&tran)?;
            tran.commit()?;
            a0
        };
        {
            let tran = conn.transaction()?;
            a0.name = String::from("test_account_1");
            a0.update(&tran)?;
            tran.commit()?;
            let tran = conn.transaction()?;
            let res = Account::by_id(a0.id, &tran)?.expect("no account");
            assert_eq!(a0.name, res.name);
        }
        {
            let tran = conn.transaction()?;
            a0.alias = String::from("alias2");
            a0.update(&tran)?;
            tran.commit()?;
            let tran = conn.transaction()?;
            let res = Account::by_id(a0.id, &tran)?.expect("no account");
            assert_eq!(a0.alias, res.alias);
        }
        {
            let tran = conn.transaction()?;
            a0.kind = AccountKind::TFSA;
            a0.update(&tran)?;
            tran.commit()?;
            let tran = conn.transaction()?;
            let res = Account::by_id(a0.id, &tran)?.expect("no account");
            assert_eq!(a0.kind, res.kind);
        }

        Ok(())
    }

    #[test]
    fn test_delete() -> Result<(), ServerError> {
        use database::asset::AssetId;
        use database::transaction::{Transaction, TxnAction};
        let mut conn = Connection::open_in_memory()?;

        let u0 = {
            let tran = conn.transaction()?;
            database::migration::run_migration(&tran)?;
            let mut u0 = User::new(
                String::from("test_user"),
                Sha256::digest("password").to_vec(),
            );
            u0.id = u0.insert(&tran)?;
            tran.commit()?;
            u0
        };
        let (a0, t0) = {
            let tran = conn.transaction()?;
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
            (a0, t0)
        };
        {
            let tran = conn.transaction()?;
            Account::delete(a0.id, &tran)?;
            tran.commit()?;

            let tran = conn.transaction()?;
            assert_eq!(None, Transaction::by_id(t0.id, &tran)?);
            assert_eq!(None, Account::by_id(a0.id, &tran)?);
        }
        Ok(())
    }
}
