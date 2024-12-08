use rusqlite::types::{FromSql, FromSqlError, ValueRef};
use serde::de::Error;
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

impl TryFrom<String> for AssetId {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let mut iter = value.split(":");
        let kind = iter.next().ok_or(())?;
        let symbol = iter.next().ok_or(())?;

        match kind {
            "CURRENCY" => Ok(Self::currency(symbol)),
            "CRYPTO" => Ok(Self::crypto(symbol)),
            "UNKNOWN" => Ok(Self::unknown(symbol)),
            s if s.starts_with("X") => Ok(Self::stock(&s[1..s.len()], symbol)),
            _ => Err(()),
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
        AssetId::try_from(value)
            .map_err(|_| D::Error::custom("Unable to convert string"))
    }
}

impl From<AssetId> for sea_query::value::Value {
    fn from(value: AssetId) -> Self {
        String::from(value).into()
    }
}

impl FromSql for AssetId {
    fn column_result(value: ValueRef<'_>) -> Result<Self, FromSqlError> {
        match value.as_str() {
            Err(err) => Err(err),
            Ok(str) => {
                if let Ok(kind) = AssetId::try_from(String::from(str)) {
                    Ok(kind)
                } else {
                    Err(FromSqlError::Other(
                        format!("{} is not a valid AssetId", str).into(),
                    ))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ServerError;

    #[test]
    fn test_convert() -> Result<(), ServerError> {
        fn assert_util(value: AssetId) {
            let value2 =
                AssetId::try_from(String::from(value.clone())).unwrap();
            assert_eq!(value, value2);
        }

        assert_util(AssetId::currency("USD"));
        assert_util(AssetId::crypto("BTC"));
        assert_util(AssetId::stock("TSE", "DLR"));
        assert_util(AssetId::unknown("TDB627"));

        AssetId::try_from(String::from("INVALID_VALUE"))
            .expect_err("expect conversion failure");

        AssetId::try_from(String::from("ABCD:EFG"))
            .expect_err("expect conversion failure");

        Ok(())
    }
}
