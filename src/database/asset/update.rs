use core::str;
use chrono::{DateTime, Utc};
use rusqlite::types::{FromSql, FromSqlError, ValueRef};
use sea_query::enum_def;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ServerError;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
pub enum AssetUpdateKind {
    Price,
    Dividend,
    Split,
}

impl From<AssetUpdateKind> for sea_query::value::Value {
    fn from(value: AssetUpdateKind) -> Self {
        match value {
            AssetUpdateKind::Price => "Price".into(),
            AssetUpdateKind::Dividend => "Dividend".into(),
            AssetUpdateKind::Split => "Split".into(),
        }
    }
}

impl FromSql for AssetUpdateKind {
    fn column_result(value: ValueRef<'_>) -> Result<Self, FromSqlError> {
        if let ValueRef::Text(text) = value {
            if let Ok(s) = str::from_utf8(text) {
                match s {
                    "Price" => return Ok(Self::Price),
                    "Dividend" => return Ok(Self::Dividend),
                    "Split" => return Ok(Self::Split),
                    _ => (),
                }
            }
        }

        Err(FromSqlError::Other(
            String::from("Cannot convert to AssetUpdateKind").into(),
        ))
    }
}


#[derive(Debug, Serialize, Deserialize, Clone)]
#[enum_def]
pub(super) struct AssetUpdate {
    #[serde(default)]
    pub asset: Uuid,
    pub query: AssetUpdateKind,
    pub updated_at: DateTime<Utc>,
}

// impl AssetUpdate {
//     pub(super) fn update(&self, transaction: &mut rusqlite::Transaction) -> Result<(), ServerError> {

//     }

//     pub(super) fn update(&self, transaction: &mut rusqlite::Transaction) -> Result<(), ServerError> {

//     }
// }

