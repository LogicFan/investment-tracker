mod id;

use crate::error::ServerError;
pub use id::AssetId;
use rusqlite::{Connection, Row};
use sea_query::{enum_def, Cond, Expr, IdenStatic, Query, SqliteQueryBuilder};
use sea_query_rusqlite::RusqliteBinder;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[enum_def]
pub struct Asset {
    #[serde(default)]
    pub id: Uuid,
    pub asset_id: AssetId,
    pub name: String,
    #[serde(default)]
    pub owner: Option<Uuid>,
}

impl PartialEq for Asset {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Asset {}

impl TryFrom<&Row<'_>> for Asset {
    type Error = rusqlite::Error;

    fn try_from(value: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.get(AssetIden::Id.as_str())?,
            asset_id: value.get(AssetIden::AssetId.as_str())?,
            name: value.get(AssetIden::Name.as_str())?,
            owner: value.get(AssetIden::Owner.as_str())?,
        })
    }
}

impl Asset {
    pub fn new(
        asset_id: AssetId,
        name: impl Into<String>,
        owner: Option<Uuid>,
    ) -> Self {
        Self {
            id: Uuid::nil(),
            asset_id,
            name: name.into(),
            owner,
        }
    }

    pub fn owner(&self, connection: &mut Connection) -> Option<super::User> {
        match self.owner {
            Some(owner) => match super::User::by_id(owner, connection) {
                Ok(Some(user)) => Some(user),
                _ => None,
            },
            _ => None,
        }
    }
}

impl Asset {
    pub fn by_id(
        id: Uuid,
        connection: &mut Connection,
    ) -> Result<Option<Self>, ServerError> {
        let (query, values) = Query::select()
            .columns([
                AssetIden::Id,
                AssetIden::AssetId,
                AssetIden::Name,
                AssetIden::Owner,
            ])
            .from(AssetIden::Table)
            .and_where(Expr::col(AssetIden::Id).eq(id))
            .build_rusqlite(SqliteQueryBuilder);

        let mut statement = connection.prepare(&query)?;
        let record: Option<Result<_, rusqlite::Error>> = statement
            .query_and_then(&*values.as_params(), |row| Asset::try_from(row))?
            .next();

        Ok(record.transpose()?)
    }

    pub fn by_asset(
        asset: AssetId,
        owner: Option<Uuid>,
        connection: &mut Connection,
    ) -> Result<Option<Self>, ServerError> {
        let (query, values) = Query::select()
            .columns([
                AssetIden::Id,
                AssetIden::AssetId,
                AssetIden::Name,
                AssetIden::Owner,
            ])
            .from(AssetIden::Table)
            .and_where(Expr::col(AssetIden::AssetId).eq(asset))
            .and_where(Expr::col(AssetIden::Owner).eq(owner))
            .build_rusqlite(SqliteQueryBuilder);

        let mut statement = connection.prepare(&query)?;
        let record: Option<Result<_, rusqlite::Error>> = statement
            .query_and_then(&*values.as_params(), |row| Asset::try_from(row))?
            .next();

        Ok(record.transpose()?)
    }

    pub fn by_owner(
        owner: Uuid,
        connection: &mut Connection,
    ) -> Result<Vec<Self>, ServerError> {
        let (query, values) = Query::select()
            .columns([
                AssetIden::Id,
                AssetIden::AssetId,
                AssetIden::Name,
                AssetIden::Owner,
            ])
            .from(AssetIden::Table)
            .and_where(Expr::col(AssetIden::Owner).eq(owner))
            .build_rusqlite(SqliteQueryBuilder);

        let mut statement = connection.prepare(&query)?;
        let record: Result<Vec<_>, rusqlite::Error> = statement
            .query_and_then(&*values.as_params(), |row| Asset::try_from(row))?
            .collect();

        Ok(record?)
    }

    pub fn search(
        query: String,
        owner: Uuid,
        connection: &mut Connection,
    ) -> Result<Vec<Self>, ServerError> {
        let (query, values) = Query::select()
            .columns([
                AssetIden::Id,
                AssetIden::AssetId,
                AssetIden::Name,
                AssetIden::Owner,
            ])
            .from(AssetIden::Table)
            .cond_where(
                Cond::any()
                    .add(Expr::col(AssetIden::Owner).eq(owner))
                    .add(Expr::col(AssetIden::Owner).is_null()),
            )
            .and_where(
                Expr::col(AssetIden::AssetId).like(format!("%:{}%", query)),
            )
            .limit(10)
            .build_rusqlite(SqliteQueryBuilder);

        let mut statement = connection.prepare(&query)?;
        let record: Result<Vec<_>, rusqlite::Error> = statement
            .query_and_then(&*values.as_params(), |row| Asset::try_from(row))?
            .collect();

        Ok(record?)
    }

    pub fn insert(
        &self,
        connection: &mut Connection,
    ) -> Result<Uuid, ServerError> {
        assert!(self.id.is_nil());

        let id = Uuid::new_v4();
        let (query, values) = Query::insert()
            .into_table(AssetIden::Table)
            .columns([
                AssetIden::Id,
                AssetIden::AssetId,
                AssetIden::Name,
                AssetIden::Owner,
            ])
            .values([
                id.into(),
                self.asset_id.clone().into(),
                self.name.clone().into(),
                self.owner.into(),
            ])?
            .build_rusqlite(SqliteQueryBuilder);

        connection.execute(&query, &*values.as_params())?;
        Ok(id)
    }

    pub fn delete(id: Uuid) -> Result<(), ServerError> {
        todo!()
    }

    pub fn update_price(&self) {
        todo!()
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
            Asset::new(AssetId::currency("CAD"), "Canadian Dollar", None);
        a0.id = a0.insert(&mut connection).expect("panic");

        let a1 = Asset::by_id(a0.id, &mut connection)
            .expect("panic")
            .expect("panic");
        assert_eq!(a0.id, a1.id);
        assert_eq!(a0.asset_id, a1.asset_id);
        assert_eq!(a0.name, a1.name);
        assert_eq!(a0.name, a1.name);

        let mut a2 = Asset::new(
            AssetId::unknown("TDB2606"),
            "TD Global Tactical Monthly Income Fund - H8",
            Some(u0.id),
        );
        a2.id = a2.insert(&mut connection).expect("panic");

        let mut a3 = Asset::new(
            AssetId::unknown("TDB627"),
            "TD Dividend Income Fund - I",
            Some(u0.id),
        );
        a3.id = a3.insert(&mut connection).expect("panic");

        // let a4 = Account::by_owner(u0.id, &mut connection).expect("panic");
        // assert!(a4.contains(&a0));
        // assert!(a4.contains(&a1));
    }
}
