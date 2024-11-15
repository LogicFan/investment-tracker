use crate::database::asset::AssetId;
use core::str;
use rusqlite::types::{FromSql, FromSqlError, ValueRef};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

type Value = (Decimal, AssetId);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type")]
pub enum TxnAction {
    Deposit {
        value: Value,
        fee: Value,
    },
    Withdrawal {
        value: Value,
        fee: Value,
    },
    Income {
        value: Value,
        reason: String,
    },
    Fee {
        value: Value,
        reason: String,
    },
    Buy {
        asset: Value,
        cash: Value,
        fee: Value,
    },
    Sell {
        asset: Value,
        cash: Value,
        fee: Value,
    },
    Dividend {
        source: AssetId,
        value: Value,
        fee: Value,
    },
    Journal {
        source: AssetId,
        target: AssetId,
        fee: Value,
    },
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
