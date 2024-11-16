mod kind;

use crate::error::ServerError;
use crate::user::authenticate;
use core::str;
pub use kind::AccountKind;
use rusqlite::{Connection, Row};
use sea_query::{enum_def, Expr, IdenStatic, Query, SqliteQueryBuilder};
use sea_query_rusqlite::RusqliteBinder;
use serde::{Deserialize, Serialize};
use std::time::SystemTimeError;
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

    pub fn owner(&self, connection: &mut Connection) -> Option<super::User> {
        match super::User::by_id(self.owner, connection) {
            Ok(Some(user)) => Some(user),
            _ => None,
        }
    }
}

impl Account {
    pub fn by_id(
        id: Uuid,
        connection: &mut Connection,
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

        let mut statement = connection.prepare(&query)?;
        let record: Option<Result<_, rusqlite::Error>> = statement
            .query_and_then(&*values.as_params(), |row| Account::try_from(row))?
            .next();

        Ok(record.transpose()?)
    }

    pub fn by_owner(
        owner: Uuid,
        connection: &mut Connection,
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

        let mut statement = connection.prepare(&query)?;
        let record: Result<Vec<_>, rusqlite::Error> = statement
            .query_and_then(&*values.as_params(), |row| Account::try_from(row))?
            .collect();

        Ok(record?)
    }

    pub fn delete(
        id: Uuid,
        connection: &mut Connection,
    ) -> Result<(), ServerError> {
        use super::transaction::TransactionIden;
        let (query1, values1) = Query::delete()
            .from_table(TransactionIden::Table)
            .and_where(Expr::col(TransactionIden::Account).eq(id))
            .build_rusqlite(SqliteQueryBuilder);

        let (query2, values2) = Query::delete()
            .from_table(AccountIden::Table)
            .and_where(Expr::col(AccountIden::Id).eq(id))
            .build_rusqlite(SqliteQueryBuilder);

        let transaction = connection.transaction()?;
        transaction.execute(&query1, &*values1.as_params())?;
        transaction.execute(&query2, &*values2.as_params())?;
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

        connection.execute(&query, &*values.as_params())?;
        Ok(id)
    }

    pub fn update(
        &self,
        connection: &mut Connection,
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

        connection.execute(&query, &*values.as_params())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::{self, User};
    use sha2::{Digest, Sha256};

    #[test]
    fn test_insert_and_select() {
        let mut connection =
            Connection::open_in_memory().expect("fail to create database");
        database::migration::run_migration(&mut connection)
            .expect("database initialization fail");

        let mut u0 = User::new(
            String::from("test_user"),
            Sha256::digest("password").to_vec(),
        );
        u0.id = u0.insert(&mut connection).expect("panic");

        let mut a0 =
            Account::new("test_account_0", "alias", u0.id, AccountKind::NRA);
        a0.id = a0.insert(&mut connection).expect("panic");

        let mut a1 =
            Account::new("test_account_1", "alias2", u0.id, AccountKind::TFSA);
        a1.id = a1.insert(&mut connection).expect("panic");

        let a2 = Account::by_id(a0.id, &mut connection)
            .expect("panic")
            .expect("panic");
        assert_eq!(a0.id, a2.id);
        assert_eq!(a0.name, a2.name);
        assert_eq!(a0.alias, a2.alias);
        assert_eq!(a0.owner, a2.owner);
        assert_eq!(a0.kind, a2.kind);

        let a4 = Account::by_owner(u0.id, &mut connection).expect("panic");
        assert!(a4.contains(&a0));
        assert!(a4.contains(&a1));
    }

    #[test]
    fn test_no_owner() {
        let mut connection =
            Connection::open_in_memory().expect("fail to create database");
        database::migration::run_migration(&mut connection)
            .expect("database initialization fail");

        let a0 = Account::new(
            "test_account",
            "alias",
            Uuid::nil(),
            AccountKind::NRA,
        );
        a0.insert(&mut connection)
            .expect_err("insert account with invalid owner");
    }

    #[test]
    fn test_update() {
        let mut connection =
            Connection::open_in_memory().expect("fail to create database");
        database::migration::run_migration(&mut connection)
            .expect("database initialization fail");

        let mut u0 = User::new(
            String::from("test_user"),
            Sha256::digest("password").to_vec(),
        );
        u0.id = u0.insert(&mut connection).expect("panic");

        let mut a0 =
            Account::new("test_account_0", "alias", u0.id, AccountKind::NRA);
        a0.id = a0.insert(&mut connection).expect("panic");

        a0.name = String::from("test_account_1");
        a0.update(&mut connection).expect("panic");
        let a1 = Account::by_id(a0.id, &mut connection)
            .expect("panic")
            .expect("panic");
        assert_eq!(a0.name, a1.name);

        a0.alias = String::from("alias2");
        a0.update(&mut connection).expect("panic");
        let a2 = Account::by_id(a0.id, &mut connection)
            .expect("panic")
            .expect("panic");
        assert_eq!(a0.alias, a2.alias);

        a0.kind = AccountKind::TFSA;
        a0.update(&mut connection).expect("panic");
        let a3 = Account::by_id(a0.id, &mut connection)
            .expect("panic")
            .expect("panic");
        assert_eq!(a0.kind, a3.kind);
    }

    #[test]
    fn test_delete() {
        // TODO: test account attached transaction

        let mut connection =
            Connection::open_in_memory().expect("fail to create database");
        database::migration::run_migration(&mut connection)
            .expect("database initialization fail");

        let mut u0 = User::new(
            String::from("test_user"),
            Sha256::digest("password").to_vec(),
        );
        u0.id = u0.insert(&mut connection).expect("panic");

        let mut a1 =
            Account::new("test_account", "alias", u0.id, AccountKind::NRA);
        a1.id = a1.insert(&mut connection).expect("panic");
        Account::delete(a1.id, &mut connection).expect("panic");
        assert_eq!(
            None,
            Account::by_id(a1.id, &mut connection).expect("panic")
        );
    }
}
