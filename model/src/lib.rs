pub mod error;
pub mod user;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Default, PartialEq, Clone)]
#[sqlx(type_name = "role", rename_all = "lowercase")]
pub enum Role {
    #[default]
    User,
    Admin,
}
