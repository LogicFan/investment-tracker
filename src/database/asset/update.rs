use crate::error::ServerError;
use chrono::{DateTime, Utc};
use core::str;
use rusqlite::types::{FromSql, FromSqlError, ValueRef};
use rusqlite::Transaction as SqlTransaction;
use sea_query::{enum_def, Expr, Query, SqliteQueryBuilder};
use sea_query_rusqlite::RusqliteBinder;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
pub(super) enum AssetUpdateKind {
    Price,
    Dividend,
    Split,
}

impl TryFrom<String> for AssetUpdateKind {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "Price" => Ok(Self::Price),
            "Dividend" => Ok(Self::Dividend),
            "Split" => Ok(Self::Split),
            _ => Err(()),
        }
    }
}

impl From<AssetUpdateKind> for String {
    fn from(value: AssetUpdateKind) -> Self {
        match value {
            AssetUpdateKind::Dividend => String::from("Dividend"),
            AssetUpdateKind::Price => String::from("Price"),
            AssetUpdateKind::Split => String::from("Split"),
        }
    }
}

impl From<AssetUpdateKind> for sea_query::value::Value {
    fn from(value: AssetUpdateKind) -> Self {
        String::from(value).into()
    }
}

impl FromSql for AssetUpdateKind {
    fn column_result(value: ValueRef<'_>) -> Result<Self, FromSqlError> {
        AssetUpdateKind::try_from(String::from(value.as_str()?)).map_err(|_| {
            FromSqlError::Other(
                String::from("Cannot convert to AssetUpdateKind").into(),
            )
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[enum_def]
pub(super) struct AssetUpdate {
    pub asset: Uuid,
    pub query: AssetUpdateKind,
    pub updated_at: (),
}

impl AssetUpdate {
    pub(super) fn new(asset: Uuid, query: AssetUpdateKind) -> Self {
        Self {
            asset,
            query,
            updated_at: (),
        }
    }

    pub(super) fn get_update(
        &self,
        transaction: &SqlTransaction,
    ) -> Result<DateTime<Utc>, ServerError> {
        let (query, values) = Query::select()
            .columns([AssetUpdateIden::UpdatedAt])
            .from(AssetUpdateIden::Table)
            .and_where(Expr::col(AssetUpdateIden::Asset).eq(self.asset))
            .and_where(Expr::col(AssetUpdateIden::Query).eq(self.query))
            .build_rusqlite(SqliteQueryBuilder);

        let mut statement = transaction.prepare(&query)?;
        let record = statement
            .query_and_then(&*values.as_params(), |row| row.get(0))?
            .next();

        Ok(record.transpose()?.unwrap_or(DateTime::<Utc>::MIN_UTC))
    }

    pub(super) fn set_update(
        &self,
        updated_at: DateTime<Utc>,
        transaction: &SqlTransaction,
    ) -> Result<(), ServerError> {
        let (query, values) = Query::insert()
            .replace()
            .into_table(AssetUpdateIden::Table)
            .columns([
                AssetUpdateIden::Asset,
                AssetUpdateIden::Query,
                AssetUpdateIden::UpdatedAt,
            ])
            .values([self.asset.into(), self.query.into(), updated_at.into()])?
            .build_rusqlite(SqliteQueryBuilder);

        transaction.execute(&query, &*values.as_params())?;
        Ok(())
    }

    pub(super) fn delete(
        asset: Uuid,
        transaction: &SqlTransaction,
    ) -> Result<(), ServerError> {
        let (query, values) = Query::delete()
            .from_table(AssetUpdateIden::Table)
            .and_where(Expr::col(AssetUpdateIden::Asset).eq(asset))
            .build_rusqlite(SqliteQueryBuilder);

        transaction.execute(&query, &*values.as_params())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database;
    use crate::database::asset::{Asset, AssetId};
    use rusqlite::Connection;

    #[test]
    fn test_convert() -> Result<(), ServerError> {
        fn assert_util(value: AssetUpdateKind) {
            let value2 =
                AssetUpdateKind::try_from(String::from(value.clone())).unwrap();
            assert_eq!(value, value2);
        }

        assert_util(AssetUpdateKind::Dividend);
        assert_util(AssetUpdateKind::Split);
        assert_util(AssetUpdateKind::Price);

        AssetUpdateKind::try_from(String::from("INVALID_VALUE"))
            .expect_err("expect conversion failure");

        Ok(())
    }

    #[test]
    fn test_set_and_get() -> Result<(), ServerError> {
        let mut conn = Connection::open_in_memory()?;

        let a0 = {
            let tran = conn.transaction()?;
            database::migration::run_migration(&tran)?;
            let mut a0 = Asset::new(
                AssetId::currency("USD"),
                "United States Dollar",
                None,
            );
            a0.id = a0.insert(&tran)?;
            tran.commit()?;
            a0
        };
        let up = AssetUpdate::new(a0.id, AssetUpdateKind::Dividend);
        {
            let tran = conn.transaction()?;
            let t = up.get_update(&tran)?;
            assert_eq!(t, DateTime::<Utc>::MIN_UTC);
        }
        {
            let tran = conn.transaction()?;
            let t = Utc::now();
            up.set_update(t, &tran)?;
            tran.commit()?;
            let tran = conn.transaction()?;
            let t2 = up.get_update(&tran)?;
            assert_eq!(t, t2);
        }

        Ok(())
    }

    #[test]
    fn test_delete() -> Result<(), ServerError> {
        let mut conn = Connection::open_in_memory()?;

        let a0 = {
            let tran = conn.transaction()?;
            database::migration::run_migration(&tran)?;
            let mut a0 = Asset::new(
                AssetId::currency("USD"),
                "United States Dollar",
                None,
            );
            a0.id = a0.insert(&tran)?;
            tran.commit()?;
            a0
        };
        let up = AssetUpdate::new(a0.id, AssetUpdateKind::Dividend);
        {
            let tran = conn.transaction()?;
            let t = Utc::now();
            up.set_update(t, &tran)?;
            tran.commit()?;
            let tran = conn.transaction()?;
            let t2 = up.get_update(&tran)?;
            assert_eq!(t, t2);
        }
        {
            let tran = conn.transaction()?;
            AssetUpdate::delete(up.asset, &tran)?;
            let t = up.get_update(&tran)?;
            assert_eq!(t, DateTime::<Utc>::MIN_UTC)
        }
        Ok(())
    }
}
