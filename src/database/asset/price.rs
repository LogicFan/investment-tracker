use super::AssetId;
use chrono::NaiveDate;
use rusqlite::{Connection, Row};
use rust_decimal::Decimal;
use sea_query::{enum_def, IdenStatic};
use serde::{Deserialize, Serialize};
use std::usize;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[enum_def]
pub struct AssetPrice {
    #[serde(default)]
    pub asset: Uuid,
    pub date: NaiveDate,
    pub price: Decimal,
    pub currency: AssetId,
}

impl TryFrom<&Row<'_>> for AssetPrice {
    type Error = rusqlite::Error;

    fn try_from(value: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            asset: value.get(AssetPriceIden::Asset.as_str())?,
            date: value.get(AssetPriceIden::Date.as_str())?,
            price: Decimal::deserialize(
                value.get(AssetPriceIden::Price.as_str())?,
            ),
            currency: value.get(AssetPriceIden::Currency.as_str())?,
        })
    }
}

impl AssetPrice {
    pub fn asset(&self, connection: &mut Connection) -> Option<super::Asset> {
        match super::Asset::by_id(self.asset, connection) {
            Ok(Some(asset)) => Some(asset),
            _ => None,
        }
    }
}
