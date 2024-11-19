use core::str;
use rusqlite::types::{FromSql, FromSqlError, ValueRef};
use serde::{Deserialize, Serialize, Serializer};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AssetId {
    // stock or ETF, anything tradable through stock exchanges
    STOCK { exchange: String, ticker: String },
    // fiat currency backed by a sovereign state government.
    CURRENCY(String),
    // crypto currency
    CRYPTO(String),
    // anything else
    UNKNOWN(String),
}

impl AssetId {
    pub fn stock(
        exchange: impl Into<String>,
        ticker: impl Into<String>,
    ) -> Self {
        return Self::STOCK {
            exchange: exchange.into(),
            ticker: ticker.into(),
        };
    }

    pub fn currency(symbol: impl Into<String>) -> Self {
        return Self::CURRENCY(symbol.into());
    }

    pub fn crypto(symbol: impl Into<String>) -> Self {
        return Self::CRYPTO(symbol.into());
    }

    pub fn unknown(symbol: impl Into<String>) -> Self {
        return Self::UNKNOWN(symbol.into());
    }
}

impl From<String> for AssetId {
    fn from(value: String) -> Self {
        let mut iter = value.split(":");
        let split: [&str; 2] = [(); 2].map(|_| iter.next().unwrap());
        let kind = split[0];
        let symbol = split[1];

        match kind {
            "CURRENCY" => Self::currency(symbol),
            "CRYPTO" => Self::crypto(symbol),
            "UNKNOWN" => Self::unknown(symbol),
            s => Self::stock(&s[1..s.len()], symbol),
        }
    }
}

impl From<AssetId> for String {
    fn from(value: AssetId) -> Self {
        match value {
            AssetId::STOCK { exchange, ticker } => {
                format!("X{}:{}", exchange, ticker)
            }
            AssetId::CURRENCY(symbol) => format!("CURRENCY:{}", symbol),
            AssetId::CRYPTO(symbol) => format!("CRYPTO:{}", symbol),
            AssetId::UNKNOWN(symbol) => format!("UNKNOWN:{}", symbol),
        }
    }
}

impl Serialize for AssetId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value: String = self.clone().into();
        value.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for AssetId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Ok(AssetId::from(value))
    }
}

impl From<AssetId> for sea_query::value::Value {
    fn from(value: AssetId) -> Self {
        String::from(value).into()
    }
}

impl FromSql for AssetId {
    fn column_result(value: ValueRef<'_>) -> Result<Self, FromSqlError> {
        if let ValueRef::Text(text) = value {
            if let Ok(s) = str::from_utf8(text) {
                return Ok(AssetId::from(String::from(s)).into());
            }
        }

        Err(FromSqlError::InvalidType)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ServerError;

    #[test]
    fn test_serde() -> Result<(), ServerError> {
        fn assert_self(asset_id: AssetId) {
            assert_eq!(asset_id.clone(), AssetId::from(String::from(asset_id)));
        }

        assert_self(AssetId::currency("USD"));
        assert_self(AssetId::crypto("BTC"));
        assert_self(AssetId::stock("TSE", "DLR"));
        assert_self(AssetId::unknown("TDB627"));

        Ok(())
    }
}
