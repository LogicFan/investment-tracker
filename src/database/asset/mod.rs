mod history;
mod id;
mod price;
mod update;

use crate::error::ServerError;
use chrono::NaiveDate;
use history::{AssetDividendIden, AssetPriceIden};
pub use id::AssetId;
use rusqlite::{Connection, Row};
use rust_decimal::Decimal;
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
        if let Some(tran) = connection.transaction().ok() {
            match self.owner {
                Some(owner) => match super::User::by_id(owner, &tran) {
                    Ok(Some(user)) => Some(user),
                    _ => None,
                },
                _ => None,
            }
        } else {
            None
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
            .and_where(
                owner
                    .map(|x| Expr::col(AssetIden::Owner).eq(x))
                    .unwrap_or(Expr::col(AssetIden::Owner).is_null()),
            )
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
        query: impl Into<String>,
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
                Expr::col(AssetIden::AssetId)
                    .like(format!("%:{}%", query.into())),
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

    pub fn delete(
        id: Uuid,
        connection: &mut Connection,
    ) -> Result<(), ServerError> {
        // TODO: delete related tables
        let (query1, values1) = Query::delete()
            .from_table(AssetIden::Table)
            .and_where(Expr::col(AssetIden::Id).eq(id))
            .build_rusqlite(SqliteQueryBuilder);

        let transaction = connection.transaction()?;
        transaction.execute(&query1, &*values1.as_params())?;
        transaction.commit()?;
        Ok(())
    }

    pub fn insert_price(
        &self,
        data: &Vec<(NaiveDate, Decimal, AssetId)>,
        connection: &mut Connection,
    ) -> Result<(), ServerError> {
        // TODO: update AssetUpdate datetime
        let mut builder = Query::insert()
            .replace()
            .into_table(AssetPriceIden::Table)
            .columns([
                AssetPriceIden::Asset,
                AssetPriceIden::Date,
                AssetPriceIden::Price,
                AssetPriceIden::Currency,
            ])
            .to_owned();
        data.iter().for_each(|(date, price, currency)| {
            _ = builder.values([
                self.id.into(),
                date.clone().into(),
                price.serialize()[..].into(),
                currency.clone().into(),
            ]);
        });
        let (query, values) = builder.build_rusqlite(SqliteQueryBuilder);

        connection.execute(&query, &*values.as_params())?;
        Ok(())
    }

    pub fn price(
        &self,
        date: NaiveDate,
        connection: &mut Connection,
    ) -> Result<Option<(Decimal, AssetId)>, ServerError> {
        // TODO: issue, cannot get multi-currency price
        let (query, values) = Query::select()
            .columns([AssetPriceIden::Price, AssetPriceIden::Currency])
            .from(AssetPriceIden::Table)
            .and_where(Expr::col(AssetPriceIden::Asset).eq(self.id))
            .and_where(Expr::col(AssetPriceIden::Date).lte(date))
            .limit(1)
            .build_rusqlite(SqliteQueryBuilder);

        let mut statement = connection.prepare(&query)?;
        let record: Option<Result<_, rusqlite::Error>> = statement
            .query_and_then(&*values.as_params(), |row| {
                Ok((
                    Decimal::deserialize(
                        row.get(AssetPriceIden::Price.as_str())?,
                    ),
                    row.get(AssetPriceIden::Currency.as_str())?,
                ))
            })?
            .next();

        Ok(record.transpose()?)
    }

    pub fn insert_dividend(
        &self,
        data: &Vec<(NaiveDate, Decimal, AssetId)>,
        connection: &mut Connection,
    ) -> Result<(), ServerError> {
        let mut builder = Query::insert()
            .replace()
            .into_table(AssetDividendIden::Table)
            .columns([
                AssetDividendIden::Asset,
                AssetDividendIden::Date,
                AssetDividendIden::Dividend,
                AssetDividendIden::Currency,
            ])
            .to_owned();
        data.iter().for_each(|(date, dividend, currency)| {
            _ = builder.values([
                self.id.into(),
                date.clone().into(),
                dividend.serialize()[..].into(),
                currency.clone().into(),
            ]);
        });
        let (query, values) = builder.build_rusqlite(SqliteQueryBuilder);

        connection.execute(&query, &*values.as_params())?;
        Ok(())
    }

    pub fn insert_split(
        &self,
        data: &Vec<(NaiveDate, Decimal)>,
    ) -> Result<(), ServerError> {
        todo!()
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::database::{self, User};
//     use rust_decimal_macros::dec;
//     use sha2::{Digest, Sha256};

//     macro_rules! date {
//         ($y:expr, $m:expr, $d:expr) => {
//             NaiveDate::from_ymd_opt($y, $m, $d).expect("panic")
//         };
//     }

//     #[test]
//     fn test_insert_and_select() {
//         let mut connection =
//             Connection::open_in_memory().expect("fail to create database");
//         database::migration::run_migration(&mut connection)
//             .expect("database initialization fail");

//         let mut u0 = User::new(
//             String::from("test_user"),
//             Sha256::digest("password").to_vec(),
//         );
//         u0.id = u0.insert(&mut connection).expect("panic");

//         let mut a0 =
//             Asset::new(AssetId::currency("CAD"), "Canadian Dollar", None);
//         a0.id = a0.insert(&mut connection).expect("panic");

//         let res = Asset::by_id(a0.id, &mut connection)
//             .expect("panic")
//             .expect("panic");
//         assert_eq!(a0.id, res.id);
//         assert_eq!(a0.asset_id, res.asset_id);
//         assert_eq!(a0.name, res.name);
//         assert_eq!(a0.name, res.name);

//         let res = Asset::by_asset(a0.asset_id.clone(), None, &mut connection)
//             .expect("panic")
//             .expect("panic");
//         assert_eq!(a0.id, res.id);
//         assert_eq!(a0.asset_id, res.asset_id);
//         assert_eq!(a0.name, res.name);
//         assert_eq!(a0.name, res.name);

//         let mut a1 = Asset::new(
//             AssetId::unknown("TDB2606"),
//             "TD Global Tactical Monthly Income Fund - H8",
//             Some(u0.id),
//         );
//         a1.id = a1.insert(&mut connection).expect("panic");

//         let res = Asset::by_id(a1.id, &mut connection)
//             .expect("panic")
//             .expect("panic");
//         assert_eq!(a1.id, res.id);
//         assert_eq!(a1.asset_id, res.asset_id);
//         assert_eq!(a1.name, res.name);
//         assert_eq!(a1.name, res.name);

//         let res =
//             Asset::by_asset(a1.asset_id.clone(), a1.owner, &mut connection)
//                 .expect("panic")
//                 .expect("panic");
//         assert_eq!(a1.id, res.id);
//         assert_eq!(a1.asset_id, res.asset_id);
//         assert_eq!(a1.name, res.name);
//         assert_eq!(a1.name, res.name);

//         let mut a2 = Asset::new(
//             AssetId::unknown("TDB627"),
//             "TD Dividend Income Fund - I",
//             Some(u0.id),
//         );
//         a2.id = a2.insert(&mut connection).expect("panic");

//         let res = Asset::by_owner(u0.id, &mut connection).expect("panic");
//         assert!(!res.contains(&a0));
//         assert!(res.contains(&a1));
//         assert!(res.contains(&a2));

//         let res = Asset::search("", u0.id, &mut connection).expect("panic");
//         assert!(res.contains(&a0));
//         assert!(res.contains(&a1));
//         assert!(res.contains(&a2));

//         let res =
//             Asset::search("", Uuid::nil(), &mut connection).expect("panic");
//         assert!(res.contains(&a0));
//         assert!(!res.contains(&a1));
//         assert!(!res.contains(&a2));

//         let res = Asset::search("C", u0.id, &mut connection).expect("panic");
//         assert!(res.contains(&a0));
//         assert!(!res.contains(&a1));
//         assert!(!res.contains(&a2));

//         let res = Asset::search("TDB6", u0.id, &mut connection).expect("panic");
//         assert!(!res.contains(&a0));
//         assert!(!res.contains(&a1));
//         assert!(res.contains(&a2));
//     }

//     #[test]
//     fn test_price() {
//         let mut connection =
//             Connection::open_in_memory().expect("fail to create database");
//         database::migration::run_migration(&mut connection)
//             .expect("database initialization fail");

//         let mut a0 =
//             Asset::new(AssetId::currency("CAD"), "Canadian Dollar", None);
//         a0.id = a0.insert(&mut connection).expect("panic");

//         let p0 = vec![
//             (date!(2010, 1, 1), dec!(1.1), AssetId::currency("USD")),
//             (date!(2010, 1, 2), dec!(1.2), AssetId::currency("USD")),
//             (date!(2010, 1, 3), dec!(1.3), AssetId::currency("USD")),
//             (date!(2010, 1, 5), dec!(1.4), AssetId::currency("USD")),
//             (date!(2010, 1, 7), dec!(1.5), AssetId::currency("USD")),
//         ];

//         a0.insert_price(&p0, &mut connection).expect("panic");
//         assert_eq!(None, a0.price(date!(2009, 12, 31), &mut connection).expect("panic"));
//         assert_eq!(Some((dec!(1.1), AssetId::currency("USD"))), a0.price(date!(2010, 1, 1), &mut connection).expect("panic"));
//     }

//     // TODO: test delete
// }
