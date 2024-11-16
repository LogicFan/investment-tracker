mod id;

use super::new_connection;
use crate::error::ServerError;
pub use id::AssetId;
use rusqlite::Row;
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
    pub fn by_id(id: Uuid) -> Result<Option<Self>, ServerError> {
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

        let connection = new_connection()?;
        let mut statement = connection.prepare(&query)?;
        let record: Option<Result<_, rusqlite::Error>> = statement
            .query_and_then(&*values.as_params(), |row| Asset::try_from(row))?
            .next();

        Ok(record.transpose()?)
    }

    pub fn by_asset(
        asset: AssetId,
        owner: Option<Uuid>,
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

        let connection = new_connection()?;
        let mut statement = connection.prepare(&query)?;
        let record: Option<Result<_, rusqlite::Error>> = statement
            .query_and_then(&*values.as_params(), |row| Asset::try_from(row))?
            .next();

        Ok(record.transpose()?)
    }

    // TODO: add by_owner

    pub fn search(
        query: String,
        owner: Uuid,
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

        let connection = new_connection()?;
        let mut statement = connection.prepare(&query)?;
        let record: Result<Vec<_>, rusqlite::Error> = statement
            .query_and_then(&*values.as_params(), |row| Asset::try_from(row))?
            .collect();

        Ok(record?)
    }

    pub fn insert(&self) -> Result<Uuid, ServerError> {
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

        let connection = new_connection()?;
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
