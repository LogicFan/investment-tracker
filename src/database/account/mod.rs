mod kind;

use super::DATABASE;
use crate::error::ServerError;
use core::str;
pub use kind::AccountKind;
use rusqlite::{Connection, Row};
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

    pub fn owner(&self) -> Option<super::User> {
        match super::User::by_id(self.owner) {
            Ok(Some(user)) => Some(user),
            _ => None,
        }
    }
}

impl Account {
    pub fn by_id(id: Uuid) -> Result<Option<Account>, ServerError> {
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
            .query_and_then(&*values.as_params(), |row| Account::try_from(row))?
            .next();

        Ok(record.transpose()?)
    }

    pub fn by_owner(owner: Uuid) -> Result<Vec<Account>, ServerError> {
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

        let connection = Connection::open(DATABASE)?;
        let mut statement = connection.prepare(&query)?;
        let record: Result<Vec<_>, rusqlite::Error> = statement
            .query_and_then(&*values.as_params(), |row| Account::try_from(row))?
            .collect();

        Ok(record?)
    }

    pub fn insert(&self) -> Result<Uuid, ServerError> {
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

        let connection = Connection::open(DATABASE)?;
        connection.execute(&query, &*values.as_params())?;
        Ok(id)
    }

    pub fn update(&self) -> Result<(), ServerError> {
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

        let connection = Connection::open(DATABASE)?;
        connection.execute(&query, &*values.as_params())?;
        Ok(())
    }

    pub fn delete(id: Uuid) -> Result<(), ServerError> {
        use super::transaction::TransactionIden;
        let (query1, values1) = Query::delete()
            .from_table(TransactionIden::Table)
            .and_where(Expr::col(TransactionIden::Account).eq(id))
            .build_rusqlite(SqliteQueryBuilder);

        let (query2, values2) = Query::delete()
            .from_table(AccountIden::Table)
            .and_where(Expr::col(AccountIden::Id).eq(id))
            .build_rusqlite(SqliteQueryBuilder);

        let mut connection = Connection::open(DATABASE)?;
        let transaction = connection.transaction()?;
        transaction.execute(&query1, &*values1.as_params())?;
        transaction.execute(&query2, &*values2.as_params())?;
        transaction.commit()?;
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
        database::init().expect("database initialization fail");

        let mut u0 = User::new(
            String::from("test_user_a0"),
            Sha256::digest("password").to_vec(),
        );
        u0.id = u0.insert().expect("panic");

        let mut a1 =
            Account::new("test_acct_a0", "alias", u0.id, AccountKind::NRA);
        a1.id = a1.insert().expect("panic");

        let mut a2 =
            Account::new("test_acct_a1", "alias2", u0.id, AccountKind::TFSA);
        a2.id = a2.insert().expect("panic");

        let a3 = Account::by_id(a1.id).expect("panic").expect("panic");
        assert_eq!(a1.id, a3.id);
        assert_eq!(a1.name, a3.name);
        assert_eq!(a1.alias, a3.alias);
        assert_eq!(a1.owner, a3.owner);
        assert_eq!(a1.kind, a3.kind);

        let a4 = Account::by_owner(u0.id).expect("panic");
        assert!(a4.contains(&a1));
        assert!(a4.contains(&a2));

        // clean up
        User::delete(u0.id).expect("test clean-up fail");
    }

    #[test]
    fn test_no_owner() {
        database::init().expect("database initialization fail");

        let a1 =
            Account::new("test_acct_a0", "alias", Uuid::nil(), AccountKind::NRA);
        a1.insert().expect_err("insert account with invalid owner");
    }

    #[test]
    fn test_update() {
        database::init().expect("database initialization fail");

        let mut u0 = User::new(
            String::from("test_user_a1"),
            Sha256::digest("password").to_vec(),
        );
        u0.id = u0.insert().expect("panic");

        let mut a1 =
            Account::new("test_acct_a0", "alias", u0.id, AccountKind::NRA);
        a1.id = a1.insert().expect("panic");

        a1.name = String::from("test_acct_a1");
        a1.update().expect("panic");
        let a2 = Account::by_id(a1.id).expect("panic").expect("panic");
        assert_eq!(a1.name, a2.name);

        a1.alias = String::from("alias2");
        a1.update().expect("panic");
        let a2 = Account::by_id(a1.id).expect("panic").expect("panic");
        assert_eq!(a1.alias, a2.alias);

        a1.kind = AccountKind::TFSA;
        a1.update().expect("panic");
        let a2 = Account::by_id(a1.id).expect("panic").expect("panic");
        assert_eq!(a1.kind, a2.kind);

        // clean up
        User::delete(u0.id).expect("test clean-up fail");
    }

    #[test]
    fn test_delete() {
        database::init().expect("database initialization fail");

        let mut u0 = User::new(
            String::from("test_user_a2"),
            Sha256::digest("password").to_vec(),
        );
        u0.id = u0.insert().expect("panic");

        let mut a1 =
            Account::new("test_acct_a0", "alias", u0.id, AccountKind::NRA);
        a1.id = a1.insert().expect("panic");
        Account::delete(a1.id).expect("panic");
        assert_eq!(None, Account::by_id(a1.id).expect("panic"));

        let mut a2 =
            Account::new("test_acct_a1", "alias", u0.id, AccountKind::NRA);
        a2.id = a2.insert().expect("panic");
        User::delete(u0.id).expect("panic");
        assert_eq!(None, Account::by_id(a2.id).expect("panic"));
    }
}
