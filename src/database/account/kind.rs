use core::str;
use rusqlite::types::{FromSql, FromSqlError, ValueRef};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
pub enum AccountKind {
    NRA,
    TFSA,
    RRSP,
    FHSA,
}

impl From<AccountKind> for sea_query::value::Value {
    fn from(value: AccountKind) -> Self {
        match value {
            AccountKind::NRA => "NRA".into(),
            AccountKind::TFSA => "TFSA".into(),
            AccountKind::RRSP => "RRSP".into(),
            AccountKind::FHSA => "FHSA".into(),
        }
    }
}

impl FromSql for AccountKind {
    fn column_result(value: ValueRef<'_>) -> Result<Self, FromSqlError> {
        if let ValueRef::Text(text) = value {
            if let Ok(s) = str::from_utf8(text) {
                match s {
                    "NRA" => return Ok(Self::NRA),
                    "TFSA" => return Ok(Self::TFSA),
                    "RRSP" => return Ok(Self::RRSP),
                    "FHSA" => return Ok(Self::FHSA),
                    _ => (),
                }
            }
        }

        Err(FromSqlError::Other(
            String::from("Cannot convert to AccountKind").into(),
        ))
    }
}
