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

impl TryFrom<String> for AccountKind {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "NRA" => Ok(Self::NRA),
            "TFSA" => Ok(Self::TFSA),
            "RRSP" => Ok(Self::RRSP),
            "FHSA" => Ok(Self::FHSA),
            _ => Err(()),
        }
    }
}

impl From<AccountKind> for String {
    fn from(value: AccountKind) -> Self {
        match value {
            AccountKind::NRA => String::from("NRA"),
            AccountKind::TFSA => String::from("TFSA"),
            AccountKind::RRSP => String::from("RRSP"),
            AccountKind::FHSA => String::from("FHSA"),
        }
    }
}

impl From<AccountKind> for sea_query::value::Value {
    fn from(value: AccountKind) -> Self {
        String::from(value).into()
    }
}

impl FromSql for AccountKind {
    fn column_result(value: ValueRef<'_>) -> Result<Self, FromSqlError> {
        match value.as_str() {
            Err(err) => Err(err),
            Ok(str) => {
                if let Ok(kind) = AccountKind::try_from(String::from(str)) {
                    Ok(kind)
                } else {
                    Err(FromSqlError::Other(
                        format!("{} is not a valid AccountKind", str).into(),
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
        fn assert_util(value: AccountKind) {
            let value2 = AccountKind::try_from(String::from(value)).unwrap();
            assert_eq!(value, value2);
        }

        assert_util(AccountKind::NRA);
        assert_util(AccountKind::TFSA);
        assert_util(AccountKind::RRSP);
        assert_util(AccountKind::FHSA);

        AccountKind::try_from(String::from("SOME RANDOM STRING"))
            .expect_err("expect conversion failure");

        Ok(())
    }
}
