mod database;
mod services;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum UserGroup {
    Viewer,
    Editor,
}

pub use database::init;
pub use services::all_users::handler as all_users;
pub use services::delete::handler as delete;
pub use services::login::handler as login;
pub use services::refresh::handler as refresh;
pub use services::upsert::handler as upsert;
