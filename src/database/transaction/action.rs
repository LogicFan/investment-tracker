use crate::database::asset::AssetId;
use core::str;
use rusqlite::types::{FromSql, FromSqlError, ValueRef};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

type Value = (Decimal, AssetId);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TxnAction {
    Deposit(Deposit),
    Withdrawal(Withdrawal),
    Income(Income),
    Fee(Fee),
    Buy(Buy),
    Sell(Sell),
    Dividend(Dividend),
    Journal(Journal),
}

impl From<TxnAction> for sea_query::value::Value {
    fn from(value: TxnAction) -> Self {
        serde_json::to_value(value).unwrap().into()
    }
}

impl FromSql for TxnAction {
    fn column_result(value: ValueRef<'_>) -> Result<Self, FromSqlError> {
        if let ValueRef::Text(text) = value {
            if let Ok(s) = str::from_utf8(text) {
                if let Ok(action) = serde_json::from_str(s) {
                    return Ok(action);
                }
            }
        }

        Err(FromSqlError::InvalidType)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deposit {
    pub value: Value,
    pub fee: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Withdrawal {
    pub value: Value,
    pub fee: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Income {
    pub value: Value,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fee {
    pub value: Value,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Buy {
    pub asset: Value,
    pub cash: Value,
    pub fee: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sell {
    pub asset: Value,
    pub cash: Value,
    pub fee: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dividend {
    pub source: AssetId,
    pub value: Value,
    pub fee: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Journal {
    pub source: AssetId,
    pub target: AssetId,
    pub fee: Value,
}
