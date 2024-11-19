use crate::database::User;
use crate::error::ServerError;
use crate::user::AsUser;

pub trait Authentication {
    fn auth(
        &self,
        token: &str,
        sql_transaction: &rusqlite::Transaction,
    ) -> Result<bool, ServerError>;
}

impl Authentication for User {
    fn auth(
        &self,
        token: &str,
        _: &rusqlite::Transaction,
    ) -> Result<bool, ServerError> {
        Ok(token.user_id()? == Some(self.id))
    }
}
